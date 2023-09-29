use std::sync::Arc;

use cp_microservice::{
    api::server::input::{
        input_plugin::InputPlugin,
        plugins::token_manager::{
            authenticator::authenticator::Authenticator, authorizer::authorizer::Authorizer,
            token_manager::TokenManager,
        },
    },
    core::error::Error,
    r#impl::api::server::input::token_manager::{
        auth0_token_wrapper::Auth0TokenWrapper, open_id_connect_config::OpenIdConnectConfig,
    },
};

pub async fn get_api_plugins(
    openid_connect_config: OpenIdConnectConfig,
) -> Result<Vec<Arc<dyn InputPlugin + Send + Sync>>, Error> {
    let token_manager = try_get_token_manager(openid_connect_config).await?;

    let api_plugins: Vec<Arc<dyn InputPlugin + Send + Sync>> = vec![token_manager];

    Ok(api_plugins)
}

async fn try_get_token_manager(
    openid_connect_config: OpenIdConnectConfig,
) -> Result<Arc<TokenManager>, Error> {
    let token_wrapper = Arc::new(Auth0TokenWrapper::try_new(openid_connect_config).await?);
    let authorizer = Authorizer::default();
    let authenticator = Authenticator::default();

    let token_manager = Arc::new(TokenManager::new(token_wrapper, authorizer, authenticator));

    Ok(token_manager)
}
