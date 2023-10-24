use async_channel::Sender;
use celes::Country;
use cp_core::geolocalization::address::Address;
use cp_microservice::{
    core::error::{Error, ErrorKind},
    logic::executor::{timeout_receive_storage_response, timeout_send_storage_request},
};

use std::str::FromStr;

use crate::{
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
            Some(action) => match action {
                OrganizationAction::Create {
                    country,
                    name,
                    address,
                    user_id,
                    replier,
                } => {
                    handle_create_organization(sender, country, name, address, user_id, replier)
                        .await
                }
                _ => Err(Error::new(
                    ErrorKind::LogicError,
                    "[logic.organization.create_organization] received an unexpected organization action",
                )),
            },
            None => Err(Error::new(
                ErrorKind::LogicError,
                "[logic.organization.create_organization] received 'None' as organization action",
            )),
        },
        _ => Err(Error::new(
            ErrorKind::LogicError,
            "[logic.organization.create_organization] received an unexpected logic request",
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
    let api_replier = validate_create_organization_input(api_replier, &country, &name, &address)?;

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

fn validate_create_organization_input(
    api_replier: tokio::sync::oneshot::Sender<Result<String, Error>>,
    country: &String,
    name: &String,
    address: &Address,
) -> Result<tokio::sync::oneshot::Sender<Result<String, Error>>, Error> {
    if country.is_empty() {
        let error = Error::new(
            ErrorKind::LogicError,
            "[logic.organization.validate_input] country is empty",
        );

        if let Err(_) = api_replier.send(Err(error.clone())) {
            log::warn!("failed to reply to api with an error");
        }

        return Err(error);
    }

    if name.is_empty() {
        let error = Error::new(
            ErrorKind::LogicError,
            "[logic.organization.validate_input] name is empty",
        );

        if let Err(_) = api_replier.send(Err(error.clone())) {
            log::warn!("failed to reply to api with an error");
        }

        return Err(error);
    }

    if address.country().is_empty() {
        let error = Error::new(
            ErrorKind::LogicError,
            "[logic.organization.validate_input] address country is empty",
        );

        if let Err(_) = api_replier.send(Err(error.clone())) {
            log::warn!("failed to reply to api with an error");
        }

        return Err(error);
    }

    if address.city().is_empty() {
        let error = Error::new(
            ErrorKind::LogicError,
            "[logic.organization.validate_input] address city is empty",
        );

        if let Err(_) = api_replier.send(Err(error.clone())) {
            log::warn!("failed to reply to api with an error");
        }

        return Err(error);
    }

    if address.street().is_empty() {
        let error = Error::new(
            ErrorKind::LogicError,
            "[logic.organization.validate_input] address street is empty",
        );

        if let Err(_) = api_replier.send(Err(error.clone())) {
            log::warn!("failed to reply to api with an error");
        }

        return Err(error);
    }

    if address.number().is_empty() {
        let error = Error::new(
            ErrorKind::LogicError,
            "[logic.organization.validate_input] address number is empty",
        );

        if let Err(_) = api_replier.send(Err(error.clone())) {
            log::warn!("failed to reply to api with an error");
        }

        return Err(error);
    }

    if address.postal_code().is_empty() {
        let error = Error::new(
            ErrorKind::LogicError,
            "[logic.organization.validate_input] address postal code is empty",
        );

        if let Err(_) = api_replier.send(Err(error.clone())) {
            log::warn!("failed to reply to api with an error");
        }

        return Err(error);
    }

    match Country::from_str(country.as_str()) {
        Ok(_) => (),
        Err(error) => {
            let error = Error::new(
                ErrorKind::LogicError,
                format!(
                    "[logic.organization.validate_input] invalid country specified: {}",
                    &error
                ),
            );

            if let Err(_) = api_replier.send(Err(error.clone())) {
                log::warn!("failed to reply to api with an error");
            }

            return Err(error);
        }
    };

    Ok(api_replier)
}

async fn get_admin_role_id(
    sender: &Sender<StorageRequest>,
    mut api_replier: tokio::sync::oneshot::Sender<Result<String, Error>>,
) -> Result<(tokio::sync::oneshot::Sender<Result<String, Error>>, String), Error> {
    let (storage_replier, storage_receiver) =
        tokio::sync::oneshot::channel::<Result<String, Error>>();

    let storage_request = StorageRequest::Role(Some(RoleAction::GetAdminRoleId {
        replier: storage_replier,
    }));

    api_replier = timeout_send_storage_request(
        TIMEOUT_CREATE_ORGANIZATION_IN_MILLISECONDS,
        storage_request,
        sender,
        api_replier,
    )
    .await?;

    let (api_replier, admin_role_id) = timeout_receive_storage_response(
        TIMEOUT_CREATE_ORGANIZATION_IN_MILLISECONDS,
        storage_receiver,
        api_replier,
    )
    .await?;

    Ok((api_replier, admin_role_id))
}

async fn create_organization_and_return_id(
    sender: &Sender<StorageRequest>,
    country: String,
    name: String,
    address: Address,
    mut api_replier: tokio::sync::oneshot::Sender<Result<String, Error>>,
) -> Result<(tokio::sync::oneshot::Sender<Result<String, Error>>, String), Error> {
    let (storage_replier, storage_receiver) =
        tokio::sync::oneshot::channel::<Result<String, Error>>();

    let storage_request = StorageRequest::Organization(Some(
        crate::storage::actions::organization_action::OrganizationAction::Create {
            country,
            name,
            address,
            replier: storage_replier,
        },
    ));

    api_replier = timeout_send_storage_request(
        TIMEOUT_CREATE_ORGANIZATION_IN_MILLISECONDS,
        storage_request,
        sender,
        api_replier,
    )
    .await?;

    let (api_replier, organization_id) = timeout_receive_storage_response(
        TIMEOUT_CREATE_ORGANIZATION_IN_MILLISECONDS,
        storage_receiver,
        api_replier,
    )
    .await?;

    Ok((api_replier, organization_id))
}

async fn create_member(
    sender: &Sender<StorageRequest>,
    user_id: String,
    admin_role_id: String,
    organization_id: String,
    mut api_replier: tokio::sync::oneshot::Sender<Result<String, Error>>,
) -> Result<tokio::sync::oneshot::Sender<Result<String, Error>>, Error> {
    let (storage_replier, storage_receiver) = tokio::sync::oneshot::channel::<Result<(), Error>>();

    let storage_request = StorageRequest::Member(Some(MemberAction::Create {
        user_id,
        admin_role_id,
        organization_id,
        replier: storage_replier,
    }));

    api_replier = timeout_send_storage_request(
        TIMEOUT_CREATE_ORGANIZATION_IN_MILLISECONDS,
        storage_request,
        sender,
        api_replier,
    )
    .await?;

    let (api_replier, _) = timeout_receive_storage_response(
        TIMEOUT_CREATE_ORGANIZATION_IN_MILLISECONDS,
        storage_receiver,
        api_replier,
    )
    .await?;

    Ok(api_replier)
}
