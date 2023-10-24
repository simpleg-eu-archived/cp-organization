use async_channel::Sender;
use cp_microservice::{
    api::{
        server::input::{action::extract_payload, api_action::api_action},
        shared::request::Request,
    },
    core::error::Error,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::logic::logic_request::LogicRequest;

const TIMEOUT_CREATE_INVITATION_CODE_IN_MILLISECONDS: u64 = 10000u64;

#[derive(Deserialize, Serialize)]
pub struct CreateInvitationCode {
    org_id: String,
    permissions: Vec<String>,
    roles: Vec<String>,
}

pub async fn create_invitation_code(
    request: Request,
    logic_request_sender: Sender<LogicRequest>,
) -> Result<Value, Error> {
    let payload: CreateInvitationCode = extract_payload(&request)?;

    let (replier, receiver) = tokio::sync::oneshot::channel::<Result<String, Error>>();
    let logic_action =
        crate::logic::actions::invitation_code_action::InvitationCodeAction::Create {
            org_id: payload.org_id,
            permissions: payload.permissions,
            roles: payload.roles,
            replier,
        };

    let logic_request = LogicRequest::InvitationCode(Some(logic_action));

    api_action(
        logic_request,
        logic_request_sender,
        TIMEOUT_CREATE_INVITATION_CODE_IN_MILLISECONDS,
        receiver,
    )
    .await
}
