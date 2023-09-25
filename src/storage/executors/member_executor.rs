use bson::doc;
use mongodb::Client;
use tokio::sync::oneshot::Sender;

use crate::{
    error::{Error, ErrorKind},
    storage::{
        actions::member_action::MemberAction,
        storage_details::{DATABASE, MEMBER_COLLECTION},
        storage_request::StorageRequest,
    },
};

pub async fn create_member(client: Client, request: StorageRequest) -> Result<(), Error> {
    match request {
        StorageRequest::Member(action) => match action {
            MemberAction::Create {
                user_id,
                admin_role_id,
                organization_id,
                replier,
            } => {
                handle_create_member(client, user_id, admin_role_id, organization_id, replier).await
            }
            _ => Err(Error::new(
                ErrorKind::StorageCreateMemberFailure,
                format!("received an unexpected organization action"),
            )),
        },
        _ => Err(Error::new(
            ErrorKind::StorageCreateMemberFailure,
            format!("received an unexpected storage request"),
        )),
    }
}

async fn handle_create_member(
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
            ErrorKind::StorageCreateMemberFailure,
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
