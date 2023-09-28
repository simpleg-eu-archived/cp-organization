use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Role {
    _id: ObjectId,
    name: String,
    permissions: Vec<String>,
    default_admin: Option<bool>,
    default_member: Option<bool>,
}

impl Role {
    pub fn id(&self) -> String {
        self._id.to_string()
    }
}
