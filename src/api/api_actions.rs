use std::{collections::HashMap, sync::Arc};

use cp_microservice::api::server::input::action::Action;

use crate::logic::logic_request::LogicRequest;

pub fn get_api_actions() -> HashMap<String, Action<LogicRequest>> {
    let mut actions: HashMap<String, Action<LogicRequest>> = HashMap::new();

    actions.insert(
        "create_organization".to_string(),
        Action::new(
            "create_organization".to_string(),
            Arc::new(move |request, sender| {
                Box::pin(crate::api::actions::organization::create_organization(
                    request, sender,
                ))
            }),
            vec!["token_manager".to_string()],
        ),
    );

    actions
}
