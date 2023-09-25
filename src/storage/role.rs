use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Role {
    _id: String,
    name: String,
    permissions: Vec<String>,
    default_admin: Option<bool>,
    default_member: Option<bool>,
}

impl Role {
    pub fn id(&self) -> &str {
        &self._id
    }
}
