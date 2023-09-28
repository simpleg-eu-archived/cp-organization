use cp_microservice::core::error::Error;
use tokio::sync::oneshot::Sender;

#[derive(Debug)]
pub enum RoleAction {
    GetAdminRoleId {
        replier: Sender<Result<String, Error>>,
    },
}
