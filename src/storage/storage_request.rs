use crate::storage::actions::{
    invitation_code_action::InvitationCodeAction, member_action::MemberAction,
    organization_action::OrganizationAction, role_action::RoleAction,
};

#[derive(Debug)]
pub enum StorageRequest {
    Organization(Option<OrganizationAction>),
    Role(Option<RoleAction>),
    Member(Option<MemberAction>),
    InvitationCode(Option<InvitationCodeAction>),
}
