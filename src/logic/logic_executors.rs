use std::{collections::HashMap, future::Future, mem::Discriminant, pin::Pin, sync::Arc};

use async_channel::Sender;
use cp_microservice::core::error::Error;

use crate::{logic::logic_request::LogicRequest, storage::storage_request::StorageRequest};

pub fn get_logic_executors() -> HashMap<
    Discriminant<LogicRequest>,
    Arc<
        dyn Fn(
                LogicRequest,
                Sender<StorageRequest>,
            ) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + Sync>>
            + Send
            + Sync,
    >,
> {
    let mut executors: HashMap<
        Discriminant<LogicRequest>,
        Arc<
            dyn Fn(
                    LogicRequest,
                    Sender<StorageRequest>,
                )
                    -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + Sync>>
                + Send
                + Sync,
        >,
    > = HashMap::new();

    executors.insert(
        std::mem::discriminant(&LogicRequest::Organization(None)),
        Arc::new(move |request, sender| {
            Box::pin(crate::logic::executors::organization::create_organization(
                request, sender,
            ))
        }),
    );

    executors
}
