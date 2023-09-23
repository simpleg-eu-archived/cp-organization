use crate::storage::actions::organization_action::OrganizationAction;

pub enum StorageRequest {
    Organization(OrganizationAction),
}
