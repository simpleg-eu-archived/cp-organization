use tokio::sync::oneshot::Sender;

use crate::error::Error;

pub enum MemberAction {
    Create {
        user_id: String,
        admin_role_id: String,
        organization_id: String,
        replier: Sender<Result<(), Error>>,
    },
}
