use crate::logic::actions::organization_action::OrganizationAction;

pub enum LogicRequest {
    Organization(OrganizationAction),
}
