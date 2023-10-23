use async_channel::Sender;
use cp_microservice::core::error::{Error, ErrorKind};

use crate::{
    logic::{actions::invitation_code_action::InvitationCodeAction, logic_request::LogicRequest},
    storage::storage_request::StorageRequest,
};

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
                    handle_create_invitation_code(sender, org_id, permissions, roles, replier).await
                }
                _ => Err(Error::new(ErrorKind::LogicError, "[logic.invitation_code.create_invitation_code] received an unexpected invitation code action")),
            },
            None => Err(Error::new(ErrorKind::LogicError, "[logic.invitation_code.create_invitation_code] received 'None' as invitation code action")),
        },
        _ => Err(Error::new(ErrorKind::LogicError, "[logic.invitation_code.create_invitation_code] received an unexpected logic request")),
    }
}

async fn handle_create_invitation_code(
    sender: Sender<StorageRequest>,
    org_id: String,
    permissions: Vec<String>,
    roles: Vec<String>,
    replier: tokio::sync::oneshot::Sender<Result<String, Error>>,
) -> Result<(), Error> {
    Ok(())
}
