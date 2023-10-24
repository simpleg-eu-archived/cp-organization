use bson::doc;
use cp_microservice::core::error::{Error, ErrorKind};
use mongodb::Client;

use crate::storage::storage_details::{DATABASE, INVITATION_CODE_COLLECTION};

pub async fn create_invitation_code(
    client: Client,
    code: String,
    org_id: String,
    permissions: Vec<String>,
    roles: Vec<String>,
    replier: tokio::sync::oneshot::Sender<Result<String, Error>>,
) -> Result<(), Error> {
    match client
        .database(DATABASE)
        .collection(INVITATION_CODE_COLLECTION)
        .insert_one(
            doc! {
                "code": code.clone(),
                "org_id": org_id,
                "permissions": permissions,
                "roles": roles
            },
            None,
        )
        .await
    {
        Ok(_) => (),
        Err(error) => {
            let error = Error::new(ErrorKind::StorageError, format!("[storage.invitation_code_executor.create_invitation_code] failed to insert invitation code: {}", &error));

            if let Err(_) = replier.send(Err(error.clone())) {
                log::warn!("failed to reply to logic with an error");
            }

            return Err(error);
        }
    };

    if let Err(_) = replier.send(Ok(code)) {
        log::warn!("failed to reply to logic with an ok");
    }

    return Ok(());
}
