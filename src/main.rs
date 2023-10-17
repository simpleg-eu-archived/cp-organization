use std::{
    collections::HashMap,
    io::{Error, ErrorKind},
    sync::Arc,
};

use cp_microservice::{
    api::server::input::{action::Action, input_plugin::InputPlugin},
    core::secrets::secrets_manager::SecretsManager,
    r#impl::init::{
        try_initialize_microservice, ApiInitializationPackage, LogicInitializationPackage,
    },
};

use crate::{
    api::{api_actions::get_api_actions, api_plugins::get_api_plugins},
    init::{
        get_amqp_api, get_amqp_connection_config, get_mongodb_client, get_openid_connect_config,
        get_secrets_manager,
    },
    logic::{logic_executors::get_logic_executors, logic_request::LogicRequest},
    storage::storage_request::StorageRequest,
};

pub mod api;
pub mod error;
pub mod init;
pub mod logic;
pub mod storage;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
    let secrets_manager: Arc<dyn SecretsManager> = get_secrets_manager()?;

    let amqp_connection_config = get_amqp_connection_config(&secrets_manager)?;
    let amqp_api = get_amqp_api()?;

    let openid_connect_config = get_openid_connect_config()?;

    let api_actions: HashMap<String, Action<LogicRequest>> = get_api_actions();

    let api_plugins: Vec<Arc<dyn InputPlugin + Send + Sync>> =
        match get_api_plugins(openid_connect_config).await {
            Ok(api_plugins) => api_plugins,
            Err(error) => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("failed to get API plugins: {}", &error),
                ))
            }
        };

    let api_initialization_package = ApiInitializationPackage::<LogicRequest> {
        amqp_connection_config,
        amqp_api,
        actions: api_actions,
        plugins: api_plugins,
    };

    let logic_executors = get_logic_executors();

    let (storage_request_sender, storage_request_receiver) =
        async_channel::bounded::<StorageRequest>(1024usize);

    let logic_initialization_package = LogicInitializationPackage::<LogicRequest, StorageRequest> {
        executors: logic_executors,
        storage_request_sender,
    };

    match try_initialize_microservice(api_initialization_package, logic_initialization_package)
        .await
    {
        Ok(_) => (),
        Err(error) => return Err(error),
    };

    let storage_connection = get_mongodb_client(&secrets_manager)?;

    let storage_dispatch = crate::storage::dispatch::Dispatch::new(
        storage_request_receiver.clone(),
        storage_connection.clone(),
    );

    storage_dispatch.run().await;

    Ok(())
}
