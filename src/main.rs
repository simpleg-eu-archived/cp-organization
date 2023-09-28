use std::{collections::HashMap, future::Future, mem::Discriminant, pin::Pin, sync::Arc};

use async_channel::Sender;
use cp_microservice::{
    api::server::{action::Action, input::input_plugin::InputPlugin},
    core::error::Error,
    r#impl::init::{
        try_initialize_microservice, ApiInitializationPackage, LogicInitializationPackage,
    },
};
use mongodb::{options::ClientOptions, Client};

use crate::{
    api::api_actions::get_api_actions,
    logic::{logic_executors::get_logic_executors, logic_request::LogicRequest},
    storage::storage_request::StorageRequest,
};

pub mod api;
pub mod error;
pub mod logic;
pub mod storage;

#[tokio::main]
pub async fn main() -> Result<(), std::io::Error> {
    let mut args = std::env::args();

    let amqp_connection_file = match args.nth(1) {
        Some(amqp_connection_file) => amqp_connection_file,
        None => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "no amqp connection file provided",
            ));
        }
    };

    let mongodb_connection_file = match args.nth(0) {
        Some(mongodb_connection_file) => mongodb_connection_file,
        None => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "no mongodb connection file provided",
            ));
        }
    };

    let amqp_api_file = match args.nth(0) {
        Some(amqp_api_file) => amqp_api_file,
        None => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "no amqp api file provided",
            ));
        }
    };

    let api_actions: HashMap<String, Action<LogicRequest>> = get_api_actions();

    let api_plugins: Vec<Arc<dyn InputPlugin + Send + Sync>> = vec![];

    let api_initialization_package = ApiInitializationPackage::<LogicRequest> {
        amqp_connection_file,
        actions: api_actions,
        plugins: api_plugins,
        amqp_api_file,
    };

    let logic_executors = get_logic_executors();

    let (storage_request_sender, storage_request_receiver) =
        async_channel::bounded::<StorageRequest>(1024usize);

    let logic_initialization_package = LogicInitializationPackage::<LogicRequest, StorageRequest> {
        executors: logic_executors,
        storage_request_sender,
    };

    let storage_connection = match get_mongodb_client_options(mongodb_connection_file) {
        Ok(mongodb_client_options) => match Client::with_options(mongodb_client_options) {
            Ok(mongodb_client) => mongodb_client,
            Err(error) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("failed to create MongoDB client: {}", &error),
                ))
            }
        },
        Err(error) => return Err(error),
    };

    match try_initialize_microservice(api_initialization_package, logic_initialization_package)
        .await
    {
        Ok(_) => (),
        Err(error) => return Err(error),
    };

    let storage_dispatch = crate::storage::dispatch::Dispatch::new(
        storage_request_receiver.clone(),
        storage_connection.clone(),
    );

    storage_dispatch.run().await;

    Ok(())
}

fn get_mongodb_client_options(
    mongodb_connection_file: String,
) -> Result<ClientOptions, std::io::Error> {
    let mongodb_connection_file_content = std::fs::read_to_string(mongodb_connection_file)?;

    let mongodb_client_options =
        match serde_json::from_str::<ClientOptions>(&mongodb_connection_file_content) {
            Ok(mongodb_client_options) => mongodb_client_options,
            Err(error) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("failed to deserialize MongoDB connection file: {}", &error),
                ))
            }
        };

    Ok(mongodb_client_options)
}
