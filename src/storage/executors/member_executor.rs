use bson::doc;
use cp_microservice::core::error::{Error, ErrorKind};
use mongodb::Client;
use tokio::sync::oneshot::Sender;

use crate::storage::{
    actions::member_action::MemberAction,
    storage_details::{DATABASE, MEMBER_COLLECTION},
    storage_request::StorageRequest,
};

pub async fn create_member(
    client: Client,
    user_id: String,
    admin_role_id: String,
    organization_id: String,
    replier: Sender<Result<(), Error>>,
) -> Result<(), Error> {
    if let Err(error) = client
        .database(DATABASE)
        .collection(MEMBER_COLLECTION)
        .insert_one(
            doc! {
                "user_id": user_id,
                "organization_id": organization_id,
                "roles": vec![admin_role_id]
            },
            None,
        )
        .await
    {
        let error = Error::new(
            ErrorKind::StorageError,
            format!(
                "[storage.member_executor.handle_create_member] failed to insert member: {}",
                &error
            ),
        );

        if let Err(_) = replier.send(Err(error.clone())) {
            log::warn!("failed to reply to logic with an error");
        }

        return Err(error);
    }

    if let Err(_) = replier.send(Ok(())) {
        log::warn!("failed to reply to logic with an ok");
    }

    Ok(())
}
