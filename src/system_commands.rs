use spatialos_sdk::worker::commands::CreateEntityRequest;
use spatialos_sdk::worker::connection::{Connection, WorkerConnection};
use spatialos_sdk::worker::entity::Entity as WorkerEntity;
use spatialos_sdk::worker::op::{CreateEntityResponseOp, StatusCode};
use spatialos_sdk::worker::{EntityId, RequestId};
use specs::prelude::{Resources, SystemData, Write};
use std::collections::HashMap;

pub type SystemCommandSender<'a> = Write<'a, SystemCommandSenderImpl>;

type IntermediateCallback<O> = Box<FnOnce(&Resources, O) + Send + Sync>;

pub struct SystemCommandSenderImpl {
    create_entity_callbacks:
        HashMap<RequestId<CreateEntityRequest>, IntermediateCallback<CreateEntityResponseOp>>,
    buffered_create_entity_requests: Vec<(
        NoAccessContainer<WorkerEntity>,
        IntermediateCallback<CreateEntityResponseOp>,
    )>,
}

// TODO expose parameters like timeout
impl SystemCommandSenderImpl {
    pub fn create_entity<F>(&mut self, entity: WorkerEntity, callback: F)
    where
        F: 'static + FnOnce(&Resources, Result<EntityId, StatusCode<EntityId>>) + Send + Sync,
    {
        self.buffered_create_entity_requests.push((
            NoAccessContainer::new(entity),
            Box::new(|res, response_op| {
                callback(
                    res,
                    SystemCommandSenderImpl::status_code_to_result(response_op.status_code),
                );
            }),
        ));
    }

    pub(crate) fn got_create_entity_response(res: &Resources, response_op: CreateEntityResponseOp) {
        let callback = {
            SystemCommandSender::fetch(res)
                .create_entity_callbacks
                .remove(&response_op.request_id)
        };

        match callback {
            Some(callback) => callback(res, response_op),
            None => println!("Unknown request ID: {:?}", response_op.request_id),
        }
    }

    pub(crate) fn flush_requests(&mut self, connection: &mut WorkerConnection) {
        for (entity, callback) in self.buffered_create_entity_requests.drain(..) {
            let request_id =
                connection.send_create_entity_request(entity.get_data(), None, Default::default());
            self.create_entity_callbacks.insert(request_id, callback);
        }
    }

    fn status_code_to_result<T>(status_code: StatusCode<T>) -> Result<T, StatusCode<T>> {
        match status_code {
            StatusCode::Success(response) => Ok(response),
            other => Err(other),
        }
    }
}

impl Default for SystemCommandSenderImpl {
    fn default() -> Self {
        SystemCommandSenderImpl {
            create_entity_callbacks: HashMap::new(),
            buffered_create_entity_requests: Vec::new(),
        }
    }
}

struct NoAccessContainer<T> {
    data: T,
}

impl<T> NoAccessContainer<T> {
    fn new(data: T) -> NoAccessContainer<T> {
        NoAccessContainer { data }
    }

    fn get_data(self) -> T {
        self.data
    }
}

// This is safe as the data inside cannot be accessed.
unsafe impl<T> Send for NoAccessContainer<T> {}
unsafe impl<T> Sync for NoAccessContainer<T> {}
