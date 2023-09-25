use cp_core::geolocalization::address::Address;
use tokio::sync::oneshot::Sender;

use crate::error::Error;

pub enum OrganizationAction {
    Create {
        country: String,
        name: String,
        address: Address,
        replier: Sender<Result<String, Error>>,
    },
}
