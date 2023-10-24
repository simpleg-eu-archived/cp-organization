use std::{collections::HashMap, sync::Arc};

use cp_microservice::api::server::input::action::Action;

use crate::logic::logic_request::LogicRequest;

pub fn get_api_actions() -> HashMap<String, Action<LogicRequest>> {
    let mut actions: HashMap<String, Action<LogicRequest>> = HashMap::new();

    actions.insert(
        "create_org".to_string(),
        Action::new(
            "create_org".to_string(),
            Arc::new(move |request, sender| {
                Box::pin(crate::api::actions::create_org::create_org(request, sender))
            }),
            Vec::new(),
        ),
    );

    actions.insert(
        "create_invitation_code".to_string(),
        Action::new(
            "create_invitation_code".to_string(),
            Arc::new(move |request, sender| {
                Box::pin(
                    crate::api::actions::create_invitation_code::create_invitation_code(
                        request, sender,
                    ),
                )
            }),
            Vec::new(),
        ),
    );

    actions
}
