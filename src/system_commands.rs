use crate::SystemDataFetch;
use spatialos_sdk::worker::commands::{
    CreateEntityRequest, DeleteEntityRequest, EntityQueryRequest, ReserveEntityIdsRequest,
};
use spatialos_sdk::worker::connection::{Connection, WorkerConnection};
use spatialos_sdk::worker::entity::Entity as WorkerEntity;
use spatialos_sdk::worker::op::{
    CreateEntityResponseOp, DeleteEntityResponseOp, EntityQueryResponseOp, QueryResponse,
    ReserveEntityIdsResponseOp, ReservedEntityIdRange, StatusCode,
};
use spatialos_sdk::worker::query::EntityQuery;
use spatialos_sdk::worker::EntityId as WorkerEntityId;
use spatialos_sdk::worker::RequestId;
use specs::prelude::{Resources, SystemData, Write};
use std::collections::HashMap;

pub type SystemCommandSender<'a> = Write<'a, SystemCommandSenderRes>;

type SystemCommandResult<T> = Result<T, StatusCode<T>>;

type IntermediateCallback<O> = Box<FnOnce(&Resources, O) + Send + Sync>;

pub struct SystemCommandSenderRes {
    reserve_entity_ids_callbacks: HashMap<
        RequestId<ReserveEntityIdsRequest>,
        IntermediateCallback<ReserveEntityIdsResponseOp>,
    >,
    buffered_reserve_entity_ids_requests:
        Vec<(u32, IntermediateCallback<ReserveEntityIdsResponseOp>)>,

    create_entity_callbacks:
        HashMap<RequestId<CreateEntityRequest>, IntermediateCallback<CreateEntityResponseOp>>,
    buffered_create_entity_requests: Vec<(
        NoAccessContainer<WorkerEntity>,
        Option<WorkerEntityId>,
        IntermediateCallback<CreateEntityResponseOp>,
    )>,

    delete_entity_callbacks:
        HashMap<RequestId<DeleteEntityRequest>, IntermediateCallback<DeleteEntityResponseOp>>,
    buffered_delete_entity_requests:
        Vec<(WorkerEntityId, IntermediateCallback<DeleteEntityResponseOp>)>,

    entity_query_callbacks:
        HashMap<RequestId<EntityQueryRequest>, IntermediateCallback<EntityQueryResponseOp>>,
    buffered_entity_query_requests: Vec<(EntityQuery, IntermediateCallback<EntityQueryResponseOp>)>,
}

// TODO expose parameters like timeout
impl SystemCommandSenderRes {
    pub fn reserve_entity_ids<F>(&mut self, number: u32, callback: F)
    where
        F: 'static
            + FnOnce(SystemCommandResult<ReservedEntityIdRange>, SystemDataFetch)
            + Send
            + Sync,
    {
        self.buffered_reserve_entity_ids_requests.push((
            number,
            Box::new(|res, response_op| {
                callback(
                    SystemCommandSenderRes::status_code_to_result(response_op.status_code),
                    SystemDataFetch::new(res),
                );
            }),
        ));
    }

    pub fn create_entity<F>(
        &mut self,
        entity: WorkerEntity,
        reserved_entity_id: Option<WorkerEntityId>,
        callback: F,
    ) where
        F: 'static + FnOnce(SystemCommandResult<WorkerEntityId>, SystemDataFetch) + Send + Sync,
    {
        self.buffered_create_entity_requests.push((
            NoAccessContainer::new(entity),
            reserved_entity_id,
            Box::new(|res, response_op| {
                callback(
                    SystemCommandSenderRes::status_code_to_result(response_op.status_code),
                    SystemDataFetch::new(res),
                );
            }),
        ));
    }

    pub fn delete_entity<F>(&mut self, entity_id: WorkerEntityId, callback: F)
    where
        F: 'static + FnOnce(SystemCommandResult<()>, SystemDataFetch) + Send + Sync,
    {
        self.buffered_delete_entity_requests.push((
            entity_id,
            Box::new(|res, response_op| {
                callback(
                    SystemCommandSenderRes::status_code_to_result(response_op.status_code),
                    SystemDataFetch::new(res),
                );
            }),
        ));
    }

    pub fn entity_query<F>(&mut self, query: EntityQuery, callback: F)
    where
        F: 'static + FnOnce(SystemCommandResult<QueryResponse>, SystemDataFetch) + Send + Sync,
    {
        self.buffered_entity_query_requests.push((
            query,
            Box::new(|res, response_op| {
                callback(
                    SystemCommandSenderRes::status_code_to_result(response_op.status_code),
                    SystemDataFetch::new(res),
                );
            }),
        ));
    }

    pub(crate) fn got_reserve_entity_ids_response(
        res: &Resources,
        response_op: ReserveEntityIdsResponseOp,
    ) {
        let callback = {
            SystemCommandSender::fetch(res)
                .reserve_entity_ids_callbacks
                .remove(&response_op.request_id)
        };

        match callback {
            Some(callback) => callback(res, response_op),
            None => println!("Unknown request ID: {:?}", response_op.request_id),
        }
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

    pub(crate) fn got_delete_entity_response(res: &Resources, response_op: DeleteEntityResponseOp) {
        let callback = {
            SystemCommandSender::fetch(res)
                .delete_entity_callbacks
                .remove(&response_op.request_id)
        };

        match callback {
            Some(callback) => callback(res, response_op),
            None => println!("Unknown request ID: {:?}", response_op.request_id),
        }
    }

    pub(crate) fn got_entity_query_response(res: &Resources, response_op: EntityQueryResponseOp) {
        let callback = {
            SystemCommandSender::fetch(res)
                .entity_query_callbacks
                .remove(&response_op.request_id)
        };

        match callback {
            Some(callback) => callback(res, response_op),
            None => println!("Unknown request ID: {:?}", response_op.request_id),
        }
    }

    pub(crate) fn flush_requests(&mut self, connection: &mut WorkerConnection) {
        for (number, callback) in self.buffered_reserve_entity_ids_requests.drain(..) {
            let request_id = connection.send_reserve_entity_ids_request(
                ReserveEntityIdsRequest(number),
                Default::default(),
            );
            self.reserve_entity_ids_callbacks
                .insert(request_id, callback);
        }

        for (entity, entity_id, callback) in self.buffered_create_entity_requests.drain(..) {
            let request_id = connection.send_create_entity_request(
                entity.get_data(),
                entity_id,
                Default::default(),
            );
            self.create_entity_callbacks.insert(request_id, callback);
        }

        for (entity_id, callback) in self.buffered_delete_entity_requests.drain(..) {
            let request_id = connection
                .send_delete_entity_request(DeleteEntityRequest(entity_id), Default::default());
            self.delete_entity_callbacks.insert(request_id, callback);
        }

        for (query, callback) in self.buffered_entity_query_requests.drain(..) {
            let request_id =
                connection.send_entity_query_request(EntityQueryRequest(query), Default::default());
            self.entity_query_callbacks.insert(request_id, callback);
        }
    }

    fn status_code_to_result<T>(status_code: StatusCode<T>) -> Result<T, StatusCode<T>> {
        match status_code {
            StatusCode::Success(response) => Ok(response),
            other => Err(other),
        }
    }
}

impl Default for SystemCommandSenderRes {
    fn default() -> Self {
        SystemCommandSenderRes {
            reserve_entity_ids_callbacks: HashMap::new(),
            buffered_reserve_entity_ids_requests: Vec::new(),

            create_entity_callbacks: HashMap::new(),
            buffered_create_entity_requests: Vec::new(),

            delete_entity_callbacks: HashMap::new(),
            buffered_delete_entity_requests: Vec::new(),

            entity_query_callbacks: HashMap::new(),
            buffered_entity_query_requests: Vec::new(),
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

#[test]
fn delete_entity_request_should_work() {
    use spatialos_sdk::worker::EntityId as WorkerEntityId;
    use specs::prelude::{System, World};

    let mut world = World::new();

    struct Sys;
    impl<'a> System<'a> for Sys {
        type SystemData = SystemCommandSender<'a>;
        fn run(&mut self, _sys: Self::SystemData) {}
    }

    <Sys as System>::SystemData::setup(&mut world.res);

    {
        let mut system_command_sender = <Sys as System>::SystemData::fetch(&world.res);
        system_command_sender.delete_entity(WorkerEntityId::new(5), |result, system_data| {
            let system_command_sender = system_data.fetch::<Sys>();
            assert_eq!(
                0,
                system_command_sender.buffered_delete_entity_requests.len()
            );
            assert!(result.is_ok());
        });
    }

    {
        let mut requests = {
            <Sys as System>::SystemData::fetch(&world.res)
                .buffered_delete_entity_requests
                .drain(..)
                .collect::<Vec<_>>()
        };

        for (_entity_id, callback) in requests.drain(..) {
            <Sys as System>::SystemData::fetch(&world.res)
                .delete_entity_callbacks
                .insert(RequestId::new(1), callback);
        }
    }

    SystemCommandSenderRes::got_delete_entity_response(
        &world.res,
        DeleteEntityResponseOp {
            request_id: RequestId::new(1),
            entity_id: WorkerEntityId::new(5),
            status_code: StatusCode::Success(()),
        },
    );
}
