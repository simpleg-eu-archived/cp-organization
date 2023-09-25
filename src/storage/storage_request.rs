use crate::storage::actions::{
    member_action::MemberAction, organization_action::OrganizationAction, role_action::RoleAction,
};

pub enum StorageRequest {
    Organization(OrganizationAction),
    Role(RoleAction),
    Member(MemberAction),
}
