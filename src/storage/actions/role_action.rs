use tokio::sync::oneshot::Sender;

use crate::error::Error;

pub enum RoleAction {
    GetAdminRoleId {
        replier: Sender<Result<String, Error>>,
    },
}
