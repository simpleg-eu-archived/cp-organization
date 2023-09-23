use cp_microservice::error::{Error, ErrorKind};
use mongodb::Client;
use tokio::sync::oneshot::Sender;

use crate::storage::{
    actions::organization_action::OrganizationAction, storage_request::StorageRequest,
};

const DATABASE: &str = "local";
const COLLECTION: &str = "organization";

pub async fn create_organization(client: Client, request: StorageRequest) -> Result<(), Error> {
    match request {
        StorageRequest::Organization(action) => match action {
            OrganizationAction::Create {
                country,
                name,
                address,
                user_id,
                replier,
            } => handle_create_organization(client, country, name, address, user_id, replier).await,
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
    address: String,
    user_id: String,
    replier: Sender<Result<(), crate::error::Error>>,
) -> Result<(), Error> {
    Ok(())
}
