use cp_microservice::core::error::Error;
use tokio::sync::oneshot::Sender;

#[derive(Debug)]
pub enum InvitationCodeAction {
    Create {
        org_id: String,
        permissions: Vec<String>,
        roles: Vec<String>,
        replier: Sender<Result<String, Error>>,
    },
}
