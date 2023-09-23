use async_channel::Sender;

use std::time::Duration;

use cp_microservice::error::{Error, ErrorKind};
use tokio::time::timeout;

use crate::{
    logic::{actions::organization_action::OrganizationAction, logic_request::LogicRequest},
    storage::storage_request::StorageRequest,
};

const TIMEOUT_CREATE_ORGANIZATION_IN_MILLISECONDS: u64 = 10000u64;

pub async fn create_organization(
    request: LogicRequest,
    sender: Sender<StorageRequest>,
) -> Result<(), Error> {
    match request {
        LogicRequest::Organization(action) => match action {
            OrganizationAction::Create {
                country,
                name,
                address,
                user_id,
                replier,
            } => handle_create_organization(sender, country, name, address, user_id, replier).await,
            _ => {
                return Err(Error::new(
                    ErrorKind::LogicError,
                    "received an unexpected organization action",
                ));
            }
        },
        _ => {
            return Err(Error::new(
                ErrorKind::LogicError,
                "received an unexpected logic request",
            ))
        }
    }
}

async fn handle_create_organization(
    sender: Sender<StorageRequest>,
    country: String,
    name: String,
    address: String,
    user_id: String,
    api_replier: tokio::sync::oneshot::Sender<Result<(), crate::error::Error>>,
) -> Result<(), Error> {
    let (storage_replier, storage_receiver) =
        tokio::sync::oneshot::channel::<Result<(), crate::error::Error>>();

    let storage_request = StorageRequest::Organization(
        crate::storage::actions::organization_action::OrganizationAction::Create {
            country,
            name,
            address,
            user_id,
            replier: storage_replier,
        },
    );

    match timeout(
        Duration::from_millis(TIMEOUT_CREATE_ORGANIZATION_IN_MILLISECONDS),
        sender.send(storage_request),
    )
    .await
    {
        Ok(result) => match result {
            Ok(_) => (),
            Err(error) => {
                match api_replier.send(Err(crate::error::Error::new(
                    crate::error::ErrorKind::StorageCreateOrganizationFailure,
                    "failed to create organization",
                ))) {
                    Ok(_) => (),
                    Err(error) => {
                        log::warn!("failed to send storage create organization failure to api")
                    }
                };

                return Err(Error::new(
                    ErrorKind::LogicError,
                    format!("failed to send storage request: {}", error),
                ));
            }
        },
        Err(error) => {
            match api_replier.send(Err(crate::error::Error::new(
                crate::error::ErrorKind::TimedOutStorageCreateOrganization,
                "timed out to create organization",
            ))) {
                Ok(_) => (),
                Err(error) => {
                    log::warn!("failed to send timed out storage create organization to api")
                }
            };

            return Err(Error::new(
                ErrorKind::LogicError,
                format!("timed out sending storage request: {}", error),
            ));
        }
    }

    match timeout(
        Duration::from_millis(TIMEOUT_CREATE_ORGANIZATION_IN_MILLISECONDS),
        storage_receiver,
    )
    .await
    {
        Ok(result) => match result {
            Ok(result) => match result {
                Ok(_) => {
                    match api_replier.send(Ok(())) {
                        Ok(_) => (),
                        Err(error) => log::warn!("failed to send 'Ok' result to api"),
                    }

                    return Ok(());
                }
                Err(error) => {
                    match api_replier.send(Err(crate::error::Error::new(
                        crate::error::ErrorKind::StorageCreateOrganizationFailure,
                        "storage failed to create organization",
                    ))) {
                        Ok(_) => (),
                        Err(error) => {
                            log::warn!("failed to send storage create organization failure to api")
                        }
                    };

                    return Err(Error::new(
                        ErrorKind::StorageError,
                        format!("storage failed to handle request: {}", error),
                    ));
                }
            },
            Err(error) => {
                match api_replier.send(Err(crate::error::Error::new(
                    crate::error::ErrorKind::StorageCreateOrganizationFailure,
                    "failed to receive create organization result from storage",
                ))) {
                    Ok(_) => (),
                    Err(error) => {
                        log::warn!("failed to send storage create organization failure to api")
                    }
                };

                return Err(Error::new(
                    ErrorKind::LogicError,
                    format!("failed to receive response from storage: {}", error),
                ));
            }
        },
        Err(error) => {
            match api_replier.send(Err(crate::error::Error::new(
                crate::error::ErrorKind::TimedOutStorageCreateOrganization,
                "timed out receiving response from storage",
            ))) {
                Ok(_) => (),
                Err(error) => {
                    log::warn!("failed to send timed out storage create organization to api")
                }
            };

            return Err(Error::new(
                ErrorKind::LogicError,
                format!("timed out waiting for storage response: {}", error),
            ));
        }
    }
}

#[cfg(test)]
const TIMEOUT_AFTER_MILLISECONDS: u64 = 200u64;

#[tokio::test]
pub async fn create_organization_sends_expected_storage_request() {
    use crate::logic::logic_request::LogicRequest;

    let (replier, receiver) = tokio::sync::oneshot::channel();
    let logic_request: LogicRequest = LogicRequest::Organization(
        crate::logic::actions::organization_action::OrganizationAction::Create {
            country: "".to_string(),
            name: "".to_string(),
            address: "".to_string(),
            user_id: "".to_string(),
            replier,
        },
    );

    let (sender, receiver) = async_channel::unbounded::<StorageRequest>();

    tokio::spawn(async move { create_organization(logic_request, sender).await });

    let (storage_replier, storage_receiver) = tokio::sync::oneshot::channel::<Result<(), Error>>();
    match timeout(
        Duration::from_millis(TIMEOUT_AFTER_MILLISECONDS),
        receiver.recv(),
    )
    .await
    .unwrap()
    {
        Ok(request) => match request {
            StorageRequest::Organization(action) => match action {
                crate::storage::actions::organization_action::OrganizationAction::Create {
                    country,
                    name,
                    address,
                    user_id,
                    replier: storage_replier,
                } => {
                    assert_eq!("", country);
                    assert_eq!("", name);
                    assert_eq!("", address);
                    assert_eq!("", user_id);
                }
            },
        },
        Err(error) => panic!("failed to receive storage request: {}", error),
    }
}
