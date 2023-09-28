use cp_core::geolocalization::address::Address;
use cp_microservice::core::error::Error;
use tokio::sync::oneshot::Sender;

#[derive(Debug)]
pub enum OrganizationAction {
    Create {
        country: String,
        name: String,
        address: Address,
        replier: Sender<Result<String, Error>>,
    },
}
