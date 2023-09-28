use cp_microservice::core::error::{Error, ErrorKind};
use mongodb::{bson::doc, Client};

use tokio::sync::oneshot::Sender;

use crate::storage::{
    actions::role_action::RoleAction,
    role::Role,
    storage_details::{DATABASE, ROLE_COLLECTION},
    storage_request::StorageRequest,
};

pub async fn get_admin_role_id(
    client: Client,
    replier: Sender<Result<String, Error>>,
) -> Result<(), Error> {
    let role = match client
        .database(DATABASE)
        .collection::<Role>(ROLE_COLLECTION)
        .find_one(
            Some(doc! {
                "default_admin": true
            }),
            None,
        )
        .await
    {
        Ok(role) => {
            match role {
                Some(role) => role,
                None => {
                    let error = Error::new(
                        ErrorKind::StorageError,
                        "[storage.role_executor.handle_get_admin_role_id] could not find the admin role",
                    );

                    if let Err(_) = replier.send(Err(error.clone())) {
                        log::warn!("storage, handle_get_admin_role_id, failed to reply to logic with an error");
                    }

                    return Err(error);
                }
            }
        }
        Err(error) => {
            let error = Error::new(
                ErrorKind::StorageError,
                format!("[storage.role_executor.handle_get_admin_role_id] failed to find the admin role: {}", &error),
            );

            if let Err(_) = replier.send(Err(error.clone())) {
                log::warn!(
                    "storage, handle_get_admin_role_id, failed to reply to logic with an error"
                );
            }

            return Err(error);
        }
    };

    if let Err(_) = replier.send(Ok(role.id().to_string())) {
        log::warn!("storage, handle_get_admin_role_id, failed to reply to logic with an ok");
    }

    Ok(())
}
