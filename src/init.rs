use std::{
    io::{Error, ErrorKind},
    sync::Arc,
};

use cp_microservice::{
    core::secrets::secrets_manager::SecretsManager,
    r#impl::{
        api::{
            server::input::token_manager::open_id_connect_config::OpenIdConnectConfig,
            shared::amqp_api_entry::AmqpApiEntry,
        },
        core::bitwarden_secrets_manager::BitwardenSecretsManager,
    },
};
use mongodb::{options::ClientOptions, Client};
use multiple_connections_lapin_wrapper::config::amqp_connect_config::AmqpConnectConfig;

const SECRETS_MANAGER_ACCESS_TOKEN_ENV: &str = "SECRETS_MANAGER_ACCESS_TOKEN";
const AMQP_CONNECTION_CONFIG_SECRET: &str = "CP_ORGANIZATION_AMQP_CONNECTION_SECRET";
const MONGODB_CONNECTION_CONFIG_SECRET: &str = "CP_ORGANIZATION_MONGODB_CONNECTION_SECRET";

pub fn get_secrets_manager() -> Result<Arc<dyn SecretsManager>, Error> {
    let access_token = match std::env::var(SECRETS_MANAGER_ACCESS_TOKEN_ENV) {
        Ok(access_token) => access_token,
        Err(_) => {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "no access token provided",
            ));
        }
    };

    Ok(Arc::new(BitwardenSecretsManager::new(access_token)))
}

pub fn get_amqp_connection_config(
    secrets_manager: &Arc<dyn SecretsManager>,
) -> Result<AmqpConnectConfig, Error> {
    let secret_id = match std::env::var(AMQP_CONNECTION_CONFIG_SECRET) {
        Ok(secret_id) => secret_id,
        Err(_) => {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!(
                    "failed to read secret id '{}'",
                    AMQP_CONNECTION_CONFIG_SECRET
                ),
            ));
        }
    };

    let amqp_connection_config_json = match secrets_manager.get(&secret_id) {
        Some(amqp_connection_config_json) => amqp_connection_config_json,
        None => {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!("no secret with id '{}'", &secret_id),
            ));
        }
    };

    let amqp_connection_config =
        match serde_json::from_str::<AmqpConnectConfig>(&amqp_connection_config_json) {
            Ok(amqp_connection_config) => amqp_connection_config,
            Err(error) => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("secret contains invalid amqp connection config: {}", &error),
                ));
            }
        };

    Ok(amqp_connection_config)
}

pub fn get_amqp_api() -> Result<Vec<AmqpApiEntry>, Error> {
    let amqp_api_file = match std::env::args().nth(2) {
        Some(amqp_api_file) => amqp_api_file,
        None => {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "no amqp api file provided",
            ));
        }
    };

    let amqp_api_file_content = match std::fs::read_to_string(&amqp_api_file) {
        Ok(content) => content,
        Err(error) => {
            return Err(Error::new(
                ErrorKind::NotFound,
                format!(
                    "failed to find amqp api file '{}': {}",
                    &amqp_api_file, &error
                ),
            ))
        }
    };

    let amqp_api = match serde_json::from_str::<Vec<AmqpApiEntry>>(&amqp_api_file_content) {
        Ok(amqp_api) => amqp_api,
        Err(error) => {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("failed to deserialize AMQP API file: {}", &error),
            ))
        }
    };

    Ok(amqp_api)
}

pub fn get_openid_connect_config() -> Result<OpenIdConnectConfig, Error> {
    let openid_connect_config_file = match std::env::args().nth(3) {
        Some(openid_connect_config_file) => openid_connect_config_file,
        None => {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "no openid connect config file provided",
            ));
        }
    };

    let openid_connect_config_file_content =
        match std::fs::read_to_string(&openid_connect_config_file) {
            Ok(content) => content,
            Err(error) => {
                return Err(Error::new(
                    ErrorKind::NotFound,
                    format!(
                        "failed to find openid connect config file '{}': {}",
                        &openid_connect_config_file, &error
                    ),
                ))
            }
        };

    let openid_connect_config =
        match serde_json::from_str::<OpenIdConnectConfig>(&openid_connect_config_file_content) {
            Ok(openid_connect_config) => openid_connect_config,
            Err(error) => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!(
                        "failed to deserialize OpenID Connect config file: {}",
                        &error
                    ),
                ))
            }
        };

    Ok(openid_connect_config)
}

pub fn get_mongodb_client(secrets_manager: &Arc<dyn SecretsManager>) -> Result<Client, Error> {
    let secret_id = match std::env::var(MONGODB_CONNECTION_CONFIG_SECRET) {
        Ok(secret_id) => secret_id,
        Err(_) => {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!(
                    "failed to read secret id '{}'",
                    MONGODB_CONNECTION_CONFIG_SECRET
                ),
            ));
        }
    };

    let mongodb_connection_config_json = match secrets_manager.get(&secret_id) {
        Some(mongodb_connection_config_json) => mongodb_connection_config_json,
        None => {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!("no secret with id '{}'", &secret_id),
            ));
        }
    };

    let mongodb_client_options =
        match serde_json::from_str::<ClientOptions>(&mongodb_connection_config_json) {
            Ok(mongodb_client_options) => mongodb_client_options,
            Err(error) => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!(
                        "failed to deserialize MongoDB connection config: {}",
                        &error
                    ),
                ))
            }
        };

    let mongodb_client = match Client::with_options(mongodb_client_options) {
        Ok(mongodb_client) => mongodb_client,
        Err(error) => {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("failed to create MongoDB client: {}", &error),
            ))
        }
    };

    Ok(mongodb_client)
}
