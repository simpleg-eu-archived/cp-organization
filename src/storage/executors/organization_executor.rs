use cp_core::geolocalization::address::Address;
use cp_microservice::core::error::{Error, ErrorKind};
use mongodb::{
    bson::{bson, doc},
    Client,
};
use tokio::sync::oneshot::Sender;

use crate::storage::{
    actions::organization_action::OrganizationAction,
    storage_details::{DATABASE, MEMBER_COLLECTION, ORGANIZATION_COLLECTION},
    storage_request::StorageRequest,
};

pub async fn create_organization(client: Client, request: StorageRequest) -> Result<(), Error> {
    match request {
        StorageRequest::Organization(action) => match action {
            OrganizationAction::Create {
                country,
                name,
                address,
                replier,
            } => handle_create_organization(client, country, name, address, replier).await,
            _ => Err(Error::new(
                ErrorKind::StorageError,
                format!("received an unexpected organization action"),
            )),
        },
        _ => Err(Error::new(
            ErrorKind::StorageError,
            format!("received an unexpected storage request"),
        )),
    }
}

async fn handle_create_organization(
    client: Client,
    country: String,
    name: String,
    address: Address,
    replier: Sender<Result<String, crate::error::Error>>,
) -> Result<(), Error> {
    let address_bson = match bson::to_bson(&address) {
        Ok(address_bson) => address_bson,
        Err(error) => {
            return Err(Error::new(
                ErrorKind::StorageError,
                format!("failed to serialize 'address': {}", error),
            ))
        }
    };

    let organization_id = match client
        .database(DATABASE)
        .collection(ORGANIZATION_COLLECTION)
        .insert_one(
            doc! {
                "country": &country,
                "name": &name,
                "address": address_bson
            },
            None,
        )
        .await
    {
        Ok(result) => match result.inserted_id.as_object_id() {
            Some(organization_id) => organization_id.to_string(),
            None => {
                if let Err(_) = replier.send(Err(crate::error::Error::new(
                    crate::error::ErrorKind::StorageCreateOrganizationFailure,
                    format!("failed to get organization id from entry"),
                ))) {
                    log::warn!(
                        "storage failed to reply with create organization id related error to logic"
                    );
                }

                return Err(Error::new(
                    ErrorKind::StorageError,
                    format!("failed to get organization id from entry"),
                ));
            }
        },
        Err(error) => {
            if let Err(_) = replier.send(Err(crate::error::Error::new(
                crate::error::ErrorKind::StorageCreateOrganizationFailure,
                format!("failed to insert new organization"),
            ))) {
                log::warn!(
                    "storage failed to reply with create organization related error to logic"
                );
            }

            return Err(Error::new(
                ErrorKind::StorageError,
                format!("failed to insert new organization: {}", error),
            ));
        }
    };

    if let Err(_) = replier.send(Ok(organization_id)) {
        log::warn!("storage failed to reply with organization id to logic");
    }

    Ok(())
}
