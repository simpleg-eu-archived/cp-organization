use std::time::Duration;

use cp_core::geolocalization::address::Address;
use cp_microservice::{
    api::{
        server::input::{
            action::{extract_payload, extract_user_id},
            plugins::token_manager::authenticator::authenticator,
        },
        shared::request::Request,
    },
    core::error::{Error, ErrorKind},
};

use crate::logic::{actions::organization_action::OrganizationAction, logic_request::LogicRequest};

use async_channel::Sender;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const TIMEOUT_CREATE_ORGANIZATION_IN_MILLISECONDS: u64 = 10000u64;

#[derive(Deserialize, Serialize)]
pub struct CreateOrganization {
    country: String,
    name: String,
    address: Address,
}

pub async fn create_org(
    request: Request,
    logic_request_sender: Sender<LogicRequest>,
) -> Result<Value, Error> {
    let payload: CreateOrganization = extract_payload(&request)?;

    let user_id = extract_user_id(&request)?;

    let (replier, receiver) = tokio::sync::oneshot::channel::<Result<String, Error>>();

    let logic_action = crate::logic::actions::organization_action::OrganizationAction::Create {
        country: payload.country,
        name: payload.name,
        address: payload.address,
        user_id: user_id,
        replier,
    };

    match logic_request_sender
        .send(LogicRequest::Organization(Some(logic_action)))
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

    let organization_id = match timeout(
        Duration::from_millis(TIMEOUT_CREATE_ORGANIZATION_IN_MILLISECONDS),
        receiver,
    )
    .await
    {
        Ok(result) => match result {
            Ok(result) => match result {
                Ok(organization_id) => organization_id,
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

    Ok(Value::String(organization_id))
}

#[cfg(test)]
use cp_microservice::api::shared::request_header::RequestHeader;
use tokio::time::timeout;

use crate::logic::logic_request::LogicRequest::Organization;

const TIMEOUT_AFTER_MILLISECONDS: u64 = 200u64;

impl Default for CreateOrganization {
    fn default() -> Self {
        CreateOrganization {
            country: "".to_string(),
            name: "".to_string(),
            address: Address::default(),
        }
    }
}

#[tokio::test]
pub async fn error_when_serializing_fails() {
    let request_header: RequestHeader =
        RequestHeader::new("example:action".to_string(), "token".to_string());
    let request: Request = Request::new(request_header, Value::Null);

    let (sender, _receiver) = async_channel::bounded(1024usize);

    match create_org(request, sender).await {
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
        create_org(request, sender).await.unwrap();
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
            Some(action) => match action {
                OrganizationAction::Create {
                    country,
                    name,
                    address,
                    user_id,
                    replier,
                } => {
                    assert_eq!("".to_string(), country);
                    assert_eq!("".to_string(), name);
                    assert_eq!(Address::default(), address);
                    assert_eq!(EXAMPLE_USER_ID.to_string(), user_id)
                }
                _ => panic!("unexpected 'action' type found"),
            },
            None => panic!("expected 'Some' got 'None'"),
        },
        _ => panic!("unexpected 'logic_request' type found"),
    }
}