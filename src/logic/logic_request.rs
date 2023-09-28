use crate::logic::actions::organization_action::OrganizationAction;

#[derive(Debug)]
pub enum LogicRequest {
    Organization(Option<OrganizationAction>),
}
