use std::time::Duration;

use async_channel::Sender;
use cp_microservice::{
    api::{
        server::input::action::{extract_payload, extract_user_id},
        shared::request::Request,
    },
    core::error::{Error, ErrorKind},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::time::timeout;

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

    match logic_request_sender
        .send(LogicRequest::InvitationCode(Some(logic_action)))
        .await
    {
        Ok(_) => (),
        Err(error) => {
            return Err(Error::new(
                ErrorKind::ApiError,
                format!("failed to send logic request: {}", &error),
            ))
        }
    }

    let invitation_code = match timeout(
        Duration::from_millis(TIMEOUT_CREATE_INVITATION_CODE_IN_MILLISECONDS),
        receiver,
    )
    .await
    {
        Ok(result) => match result {
            Ok(result) => match result {
                Ok(invitation_code) => invitation_code,
                Err(error) => {
                    return Err(Error::new(
                        ErrorKind::RequestError,
                        format!("failed to handle request: {}", &error),
                    ))
                }
            },
            Err(error) => {
                return Err(Error::new(
                    ErrorKind::ApiError,
                    format!("failed to receive logic result: {}", &error),
                ))
            }
        },
        Err(_) => {
            return Err(Error::new(
                ErrorKind::ApiError,
                "timed out waiting for logic result",
            ))
        }
    };

    Ok(Value::String(invitation_code))
}
