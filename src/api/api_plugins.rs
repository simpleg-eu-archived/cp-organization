use std::sync::Arc;

use cp_microservice::{api::server::input::input_plugin::InputPlugin, core::error::Error};

pub async fn get_api_plugins() -> Result<Vec<Arc<dyn InputPlugin + Send + Sync>>, Error> {
    let api_plugins: Vec<Arc<dyn InputPlugin + Send + Sync>> = vec![];

    Ok(api_plugins)
}
