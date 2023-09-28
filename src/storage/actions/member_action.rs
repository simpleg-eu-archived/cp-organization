use cp_microservice::core::error::Error;
use tokio::sync::oneshot::Sender;

#[derive(Debug)]
pub enum MemberAction {
    Create {
        user_id: String,
        admin_role_id: String,
        organization_id: String,
        replier: Sender<Result<(), Error>>,
    },
}
