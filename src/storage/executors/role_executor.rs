use bson::doc;
use mongodb::Client;

use cp_microservice::core::error::{Error, ErrorKind};
use tokio::sync::oneshot::Sender;

use crate::storage::{
    actions::role_action::RoleAction,
    role::Role,
    storage_details::{DATABASE, ROLE_COLLECTION},
    storage_request::StorageRequest,
};

pub async fn get_admin_role_id(client: Client, request: StorageRequest) -> Result<(), Error> {
    match request {
        StorageRequest::Role(action) => match action {
            RoleAction::GetAdminRoleId { replier } => {
                handle_get_admin_role_id(client, replier).await
            }
            _ => Err(Error::new(
                ErrorKind::StorageError,
                format!("received an unexpected role action"),
            )),
        },
        _ => Err(Error::new(
            ErrorKind::StorageError,
            format!("received an unexpected storage request"),
        )),
    }
}

async fn handle_get_admin_role_id(
    client: Client,
    replier: Sender<Result<String, crate::error::Error>>,
) -> Result<(), Error> {
    let role = match client
        .database(DATABASE)
        .collection::<Role>(ROLE_COLLECTION)
        .find_one(
            doc! {
                "default_admin": true
            },
            None,
        )
        .await
    {
        Ok(role) => {
            match role {
                Some(role) => role,
                None => {
                    if let Err(_) = replier.send(Err(crate::error::Error::new(
                        crate::error::ErrorKind::StorageGetAdminRoleIdFailure,
                        "could not find the admin role",
                    ))) {
                        log::warn!("storage, handle_get_admin_role_id, failed to reply to logic with an error");
                    }

                    return Err(Error::new(
                        ErrorKind::StorageError,
                        "could not find the admin role",
                    ));
                }
            }
        }
        Err(error) => {
            if let Err(_) = replier.send(Err(crate::error::Error::new(
                crate::error::ErrorKind::StorageGetAdminRoleIdFailure,
                format!("failed to find the admin role: {}", &error),
            ))) {
                log::warn!(
                    "storage, handle_get_admin_role_id, failed to reply to logic with an error"
                );
            }

            return Err(Error::new(
                ErrorKind::StorageError,
                format!("failed to find the admin role: {}", &error),
            ));
        }
    };

    if let Err(_) = replier.send(Ok(role.id().to_string())) {
        log::warn!("storage, handle_get_admin_role_id, failed to reply to logic with an ok");
    }

    Ok(())
}
