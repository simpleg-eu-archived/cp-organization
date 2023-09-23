use std::time::Duration;

use cp_microservice::{
    api::{
        server::input::plugins::token_manager::authenticator::authenticator,
        shared::request::Request,
    },
    error::{Error, ErrorKind},
};

use crate::logic::{actions::organization_action::OrganizationAction, logic_request::LogicRequest};

use async_channel::Sender;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const TIMEOUT_CREATE_ORGANIZATION_IN_MILLISECONDS: u64 = 10000u64;

#[derive(Deserialize, Serialize)]
pub struct CreateOrganization {
    country: String,
    name: String,
    address: String,
}

impl Default for CreateOrganization {
    fn default() -> Self {
        CreateOrganization {
            country: "".to_string(),
            name: "".to_string(),
            address: "".to_string(),
        }
    }
}

pub async fn create_organization(
    request: Request,
    logic_request_sender: Sender<LogicRequest>,
) -> Result<Value, Error> {
    let payload: CreateOrganization =
        match serde_json::from_value::<CreateOrganization>(request.payload().clone()) {
            Ok(payload) => payload,
            Err(error) => {
                return Err(Error::new(
                    ErrorKind::RequestError,
                    format!("invalid payload: {}", error),
                ))
            }
        };

    let user_id = match request
        .header()
        .get_extra(&authenticator::USER_ID_KEY.to_string())
    {
        Some(user_id) => user_id.clone(),
        None => {
            return Err(Error::new(
                ErrorKind::RequestError,
                "missing 'user_id' extra from request header",
            ))
        }
    };

    let (replier, receiver) = tokio::sync::oneshot::channel::<Result<(), crate::error::Error>>();

    let logic_action = crate::logic::actions::organization_action::OrganizationAction::Create {
        country: payload.country,
        name: payload.name,
        address: payload.address,
        user_id: user_id,
        replier,
    };

    match logic_request_sender
        .send(LogicRequest::Organization(logic_action))
        .await
    {
        Ok(_) => (),
        Err(error) => {
            return Err(Error::new(
                ErrorKind::ApiError,
                format!("failed to send logic request: {}", error),
            ))
        }
    }

    match timeout(
        Duration::from_millis(TIMEOUT_CREATE_ORGANIZATION_IN_MILLISECONDS),
        receiver,
    )
    .await
    {
        Ok(result) => match result {
            Ok(result) => match result {
                Ok(_) => (),
                Err(error) => {
                    return Err(Error::new(
                        ErrorKind::RequestError,
                        format!("failed to handle request: {}", error),
                    ))
                }
            },
            Err(error) => {
                return Err(Error::new(
                    ErrorKind::ApiError,
                    format!("failed to receive logic result: {}", error),
                ))
            }
        },
        Err(_error) => {
            return Err(Error::new(
                ErrorKind::ApiError,
                format!("timed out waiting for logic result"),
            ))
        }
    }

    Ok(Value::Null)
}

#[cfg(test)]
use cp_microservice::api::shared::request_header::RequestHeader;
use tokio::time::timeout;

use crate::logic::logic_request::LogicRequest::Organization;

const TIMEOUT_AFTER_MILLISECONDS: u64 = 200u64;

#[tokio::test]
pub async fn error_when_serializing_fails() {
    let request_header: RequestHeader =
        RequestHeader::new("example:action".to_string(), "token".to_string());
    let request: Request = Request::new(request_header, Value::Null);

    let (sender, _receiver) = async_channel::bounded(1024usize);

    match create_organization(request, sender).await {
        Ok(_) => panic!("expected 'Err' got 'Ok'"),
        Err(error) => assert_eq!(ErrorKind::RequestError, error.kind),
    }
}

#[tokio::test]
pub async fn sends_expected_logic_request() {
    const EXAMPLE_USER_ID: &str = "1";

    let request_header: RequestHeader =
        RequestHeader::new("example:action".to_string(), "token".to_string());
    let create_organization_payload = match serde_json::to_value(CreateOrganization::default()) {
        Ok(payload) => payload,
        Err(error) => panic!(
            "failed to serialize default CreateOrganization payload: {}",
            error
        ),
    };

    let mut request: Request = Request::new(request_header, create_organization_payload);
    request.mut_header().add_extra(
        authenticator::USER_ID_KEY.to_string(),
        EXAMPLE_USER_ID.to_string(),
    );

    let (sender, receiver) = async_channel::bounded(1024usize);

    tokio::spawn(async move {
        create_organization(request, sender).await.unwrap();
    });

    let logic_request = match timeout(
        Duration::from_millis(TIMEOUT_AFTER_MILLISECONDS),
        receiver.recv(),
    )
    .await
    .unwrap()
    {
        Ok(request) => request,
        Err(error) => panic!("failed to receive 'LogicRequest': {}", error),
    };

    let (replier, receiver) = tokio::sync::oneshot::channel::<Result<(), crate::error::Error>>();

    match logic_request {
        Organization(action) => match action {
            OrganizationAction::Create {
                country,
                name,
                address,
                user_id,
                replier,
            } => {
                assert_eq!("".to_string(), country);
                assert_eq!("".to_string(), name);
                assert_eq!("".to_string(), address);
                assert_eq!(EXAMPLE_USER_ID.to_string(), user_id)
            }
            _ => panic!("unexpected 'action' type found"),
        },
        _ => panic!("unexpected 'logic_request' type found"),
    }
}
