use crate::logic::actions::{
    invitation_code_action::InvitationCodeAction, organization_action::OrganizationAction,
};

#[derive(Debug)]
pub enum LogicRequest {
    Organization(Option<OrganizationAction>),
    InvitationCode(Option<InvitationCodeAction>),
}
