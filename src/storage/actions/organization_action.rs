use tokio::sync::oneshot::Sender;

use crate::error::Error;

pub enum OrganizationAction {
    Create {
        country: String,
        name: String,
        address: String,
        user_id: String,
        replier: Sender<Result<(), Error>>,
    },
}
