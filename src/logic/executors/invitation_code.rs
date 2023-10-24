use async_channel::Sender;
use cp_microservice::{
    core::error::{Error, ErrorKind},
    logic::executor::{timeout_receive_storage_response, timeout_send_storage_request},
};
use uuid::Uuid;

use crate::{
    logic::{actions::invitation_code_action::InvitationCodeAction, logic_request::LogicRequest},
    storage::{self, storage_request::StorageRequest},
};

const TIMEOUT_CREATE_INVITATION_CODE_IN_MILLISECONDS: u64 = 10000u64;

pub async fn create_invitation_code(
    request: LogicRequest,
    sender: Sender<StorageRequest>,
) -> Result<(), Error> {
    match request {
        LogicRequest::InvitationCode(action) => match action {
            Some(action) => match action {
                InvitationCodeAction::Create {
                    org_id,
                    permissions,
                    roles,
                    replier,
                } => {
                    handle_create_invitation_code(&sender, org_id, permissions, roles, replier).await
                }
                _ => Err(Error::new(ErrorKind::LogicError, "[logic.invitation_code.create_invitation_code] received an unexpected invitation code action")),
            },
            None => Err(Error::new(ErrorKind::LogicError, "[logic.invitation_code.create_invitation_code] received 'None' as invitation code action")),
        },
        _ => Err(Error::new(ErrorKind::LogicError, "[logic.invitation_code.create_invitation_code] received an unexpected logic request")),
    }
}

async fn handle_create_invitation_code(
    sender: &Sender<StorageRequest>,
    org_id: String,
    permissions: Vec<String>,
    roles: Vec<String>,
    mut api_replier: tokio::sync::oneshot::Sender<Result<String, Error>>,
) -> Result<(), Error> {
    let code = Uuid::new_v4().to_string();

    let (storage_replier, storage_receiver) = tokio::sync::oneshot::channel();

    let storage_request = StorageRequest::InvitationCode(Some(
        storage::actions::invitation_code_action::InvitationCodeAction::Create {
            code: code.clone(),
            org_id,
            permissions,
            roles,
            replier: storage_replier,
        },
    ));

    api_replier = timeout_send_storage_request(
        TIMEOUT_CREATE_INVITATION_CODE_IN_MILLISECONDS,
        storage_request,
        sender,
        api_replier,
    )
    .await?;

    let (api_replier, _) = timeout_receive_storage_response(
        TIMEOUT_CREATE_INVITATION_CODE_IN_MILLISECONDS,
        storage_receiver,
        api_replier,
    )
    .await?;

    if let Err(_) = api_replier.send(Ok(code)) {
        log::warn!("failed to reply to api with an ok");
    }

    Ok(())
}
