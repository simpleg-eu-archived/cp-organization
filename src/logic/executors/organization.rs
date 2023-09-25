use async_channel::Sender;
use cp_core::geolocalization::address::Address;

use std::time::Duration;

use tokio::time::timeout;

use crate::{
    error::{Error, ErrorKind},
    logic::{actions::organization_action::OrganizationAction, logic_request::LogicRequest},
    storage::{
        actions::{member_action::MemberAction, role_action::RoleAction},
        storage_request::StorageRequest,
    },
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
            _ => Err(Error::new(
                ErrorKind::LogicCreateOrganizationFailure,
                "received an unexpected organization action",
            )),
        },
        _ => Err(Error::new(
            ErrorKind::LogicCreateOrganizationFailure,
            "received an unexpected logic request",
        )),
    }
}

async fn handle_create_organization(
    sender: Sender<StorageRequest>,
    country: String,
    name: String,
    address: Address,
    user_id: String,
    api_replier: tokio::sync::oneshot::Sender<Result<String, Error>>,
) -> Result<(), Error> {
    let (api_replier, admin_role_id) = get_admin_role_id(&sender, api_replier).await?;

    let (api_replier, organization_id) =
        create_organization_and_return_id(&sender, country, name, address, api_replier).await?;

    let api_replier = create_member(
        &sender,
        user_id,
        admin_role_id.clone(),
        organization_id.clone(),
        api_replier,
    )
    .await?;

    if let Err(_) = api_replier.send(Ok(organization_id)) {
        log::warn!("failed to reply to api with an ok");
    }

    Ok(())
}

async fn get_admin_role_id(
    sender: &Sender<StorageRequest>,
    api_replier: tokio::sync::oneshot::Sender<Result<String, crate::error::Error>>,
) -> Result<
    (
        tokio::sync::oneshot::Sender<Result<String, crate::error::Error>>,
        String,
    ),
    Error,
> {
    let (storage_replier, storage_receiver) =
        tokio::sync::oneshot::channel::<Result<String, Error>>();

    let storage_request = StorageRequest::Role(RoleAction::GetAdminRoleId {
        replier: storage_replier,
    });

    match timeout(
        Duration::from_millis(TIMEOUT_CREATE_ORGANIZATION_IN_MILLISECONDS),
        sender.send(storage_request),
    )
    .await
    {
        Ok(result) => {
            if let Err(error) = result {
                let error = Error::new(
                    crate::error::ErrorKind::LogicCreateOrganizationFailure,
                    format!(
                        "[logic.organization.get_admin_role_id] failed to send storage request: {}",
                        &error
                    ),
                );

                if let Err(_) = api_replier.send(Err(error.clone())) {
                    log::warn!("failed to reply to api with an error");
                }

                return Err(error);
            }
        }
        Err(_) => {
            let error = Error::new(
                crate::error::ErrorKind::LogicCreateOrganizationTimedOut,
                "[logic.organization.get_admin_role_id] timed out sending storage request",
            );

            if let Err(_) = api_replier.send(Err(error.clone())) {
                log::warn!("failed to reply to api with an error");
            }

            return Err(error);
        }
    }

    let admin_role_id = match timeout(
        Duration::from_millis(TIMEOUT_CREATE_ORGANIZATION_IN_MILLISECONDS),
        storage_receiver,
    )
    .await
    {
        Ok(result) => match result {
            Ok(result) => match result {
                Ok(admin_role_id) => admin_role_id,
                Err(error) => {
                    if let Err(_) = api_replier.send(Err(error.clone())) {
                        log::warn!("failed to reply to api with an error");
                    }

                    return Err(error);
                }
            },
            Err(error) => {
                let error = Error::new(
                    crate::error::ErrorKind::LogicCreateOrganizationFailure,
                    format!("[logic.organization.get_admin_role_id] failed to receive response from storage: {}", &error),
                );

                if let Err(_) = api_replier.send(Err(error.clone())) {
                    log::warn!("failed to reply to api with an error");
                }

                return Err(error);
            }
        },
        Err(_) => {
            let error = Error::new(
                crate::error::ErrorKind::LogicCreateOrganizationTimedOut,
                "[logic.organization.get_admin_role_id] timed out receiving storage response",
            );

            if let Err(_) = api_replier.send(Err(error.clone())) {
                log::warn!("failed to reply to api with an error");
            }

            return Err(error);
        }
    };

    Ok((api_replier, admin_role_id))
}

async fn create_organization_and_return_id(
    sender: &Sender<StorageRequest>,
    country: String,
    name: String,
    address: Address,
    api_replier: tokio::sync::oneshot::Sender<Result<String, Error>>,
) -> Result<(tokio::sync::oneshot::Sender<Result<String, Error>>, String), Error> {
    let (storage_replier, storage_receiver) =
        tokio::sync::oneshot::channel::<Result<String, Error>>();

    let storage_request = StorageRequest::Organization(
        crate::storage::actions::organization_action::OrganizationAction::Create {
            country,
            name,
            address,
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
                let error = Error::new(
                    crate::error::ErrorKind::StorageCreateOrganizationFailure,
                    format!("[logic.organization.create_organization_and_return_id] failed to send storage request forcreate organization: {}", &error),
                );

                if let Err(_) = api_replier.send(Err(error.clone())) {
                    log::warn!("failed to reply to api with an error")
                }

                return Err(error);
            }
        },
        Err(error) => {
            let error = Error::new(
                crate::error::ErrorKind::LogicCreateOrganizationTimedOut,
                format!("[logic.organization.create_organization_and_return_id] timed out to create organization: {}", &error),
            );

            if let Err(_) = api_replier.send(Err(error.clone())) {
                log::warn!("failed to reply to api with an error")
            }

            return Err(error);
        }
    }

    let organization_id = match timeout(
        Duration::from_millis(TIMEOUT_CREATE_ORGANIZATION_IN_MILLISECONDS),
        storage_receiver,
    )
    .await
    {
        Ok(result) => match result {
            Ok(result) => match result {
                Ok(organization_id) => organization_id,
                Err(error) => {
                    let error = Error::new(
                        crate::error::ErrorKind::StorageCreateOrganizationFailure,
                        format!("[logic.organization.create_organization_and_return_id] storage failed to handle request: {}", &error),
                    );

                    if let Err(_) = api_replier.send(Err(error.clone())) {
                        log::warn!("failed to reply to api with an error");
                    }

                    return Err(error);
                }
            },
            Err(error) => {
                let error = Error::new(
                    crate::error::ErrorKind::StorageCreateOrganizationFailure,
                    format!("[logic.organization.create_organization_and_return_id] failed to receive response from storage: {}", &error),
                );

                if let Err(_) = api_replier.send(Err(error.clone())) {
                    log::warn!("failed to reply to api with an error")
                }

                return Err(error);
            }
        },
        Err(error) => {
            let error = Error::new(
                crate::error::ErrorKind::LogicCreateOrganizationTimedOut,
                format!("[logic.organization.create_organization_and_return_id] timed out receiving response from storage: {}", &error),
            );

            if let Err(_) = api_replier.send(Err(error.clone())) {
                log::warn!("failed to reply to api with an error")
            }

            return Err(Error::new(
                ErrorKind::LogicCreateOrganizationTimedOut,
                format!("[logic.organization.create_organization_and_return_id] timed out waiting for storage response: {}", &error),
            ));
        }
    };

    Ok((api_replier, organization_id))
}

async fn create_member(
    sender: &Sender<StorageRequest>,
    user_id: String,
    admin_role_id: String,
    organization_id: String,
    api_replier: tokio::sync::oneshot::Sender<Result<String, Error>>,
) -> Result<tokio::sync::oneshot::Sender<Result<String, Error>>, Error> {
    let (storage_replier, storage_receiver) = tokio::sync::oneshot::channel::<Result<(), Error>>();

    let storage_request = StorageRequest::Member(MemberAction::Create {
        user_id,
        admin_role_id,
        organization_id,
        replier: storage_replier,
    });

    match timeout(
        Duration::from_millis(TIMEOUT_CREATE_ORGANIZATION_IN_MILLISECONDS),
        sender.send(storage_request),
    )
    .await
    {
        Ok(result) => {
            if let Err(error) = result {
                let error = Error::new(
                    ErrorKind::LogicCreateOrganizationFailure,
                    format!(
                        "[logic.organization.create_member] failed to send storage request: {}",
                        &error
                    ),
                );

                if let Err(_) = api_replier.send(Err(error.clone())) {
                    log::warn!("failed to reply to api with an error")
                }

                return Err(error);
            }
        }
        Err(error) => {
            let error = Error::new(
                ErrorKind::LogicCreateOrganizationTimedOut,
                format!(
                    "[logic.organization.create_member] timed out sending storage request: {}",
                    &error
                ),
            );

            if let Err(_) = api_replier.send(Err(error.clone())) {
                log::warn!("failed to reply to api with an error")
            }

            return Err(error);
        }
    }

    match timeout(
        Duration::from_millis(TIMEOUT_CREATE_ORGANIZATION_IN_MILLISECONDS),
        storage_receiver,
    )
    .await
    {
        Ok(result) => match result {
            Ok(result) => {
                if let Err(error) = result {
                    if let Err(_) = api_replier.send(Err(error.clone())) {
                        log::warn!("failed to reply to api with an error")
                    }

                    return Err(error);
                }
            }
            Err(error) => {
                let error = Error::new(
                    ErrorKind::LogicCreateOrganizationTimedOut,
                    format!(
                        "[logic.organization.create_member] failed to receive storage response: {}",
                        &error
                    ),
                );

                if let Err(_) = api_replier.send(Err(error.clone())) {
                    log::warn!("failed to reply to api with an error")
                }

                return Err(error);
            }
        },
        Err(error) => {
            let error = Error::new(
                ErrorKind::LogicCreateOrganizationTimedOut,
                format!(
                    "[logic.organization.create_member] timed out receiving storage response: {}",
                    &error
                ),
            );

            if let Err(_) = api_replier.send(Err(error.clone())) {
                log::warn!("failed to reply to api with an error")
            }

            return Err(error);
        }
    }

    Ok(api_replier)
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
            address: Address::default(),
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
                    replier: storage_replier,
                } => {
                    assert_eq!("", country);
                    assert_eq!("", name);
                    assert_eq!(Address::default(), address);
                }
            },
            _ => panic!("received unexpected storage request"),
        },
        Err(error) => panic!("failed to receive storage request: {}", error),
    }
}
