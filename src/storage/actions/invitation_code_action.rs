use cp_microservice::core::error::Error;

#[derive(Debug)]
pub enum InvitationCodeAction {
    Create {
        code: String,
        org_id: String,
        permissions: Vec<String>,
        roles: Vec<String>,
        replier: tokio::sync::oneshot::Sender<Result<String, Error>>,
    },
}
