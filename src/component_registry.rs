use crate::commands::{
    CommandRequests, CommandRequestsExt, CommandResponder, CommandSender, CommandSenderImpl,
};
use crate::entities::SpatialEntity;
use crate::storage::{AuthorityBitSet, SpatialStorage, SpatialWriteStorage};
use crate::SynchronisedComponent;
use spatialos_sdk::worker::component::Component as SpatialComponent;
use spatialos_sdk::worker::component::ComponentId;
use spatialos_sdk::worker::connection::WorkerConnection;
use spatialos_sdk::worker::op::{
    AddComponentOp, AuthorityChangeOp, CommandRequestOp, CommandResponseOp, ComponentUpdateOp,
};
use specs::prelude::{Join, ReadStorage, Resources, Storage, SystemData, Write, WriteStorage};
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;

pub(crate) struct ComponentRegistry {
    interfaces: HashMap<ComponentId, Box<ComponentDispatcherInterface + Send + Sync>>,
}

impl ComponentRegistry {
    fn new() -> ComponentRegistry {
        ComponentRegistry {
            interfaces: HashMap::new(),
        }
    }

    pub(crate) fn register_component<T: 'static + SpatialComponent>(res: &mut Resources) {
        // Create component data storage.
        WriteStorage::<SynchronisedComponent<T>>::setup(res);

        // Create command sender resource.
        Write::<CommandSenderImpl<T>>::setup(res);

        // Create command receiver storage.
        CommandRequests::<T>::setup(res);

        res.entry::<ComponentRegistry>()
            .or_insert_with(|| ComponentRegistry::new())
            .register_component_on_self::<T>();

        res.insert(AuthorityBitSet::<T>::new());
    }

    fn register_component_on_self<T: 'static + SpatialComponent>(&mut self) {
        let interface = ComponentDispatcher::<T> {
            _phantom: PhantomData,
        };
        self.interfaces.insert(T::ID, Box::new(interface));
    }

    pub(crate) fn get_interface(
        &self,
        component_id: ComponentId,
    ) -> Option<&Box<ComponentDispatcherInterface + Send + Sync>> {
        self.interfaces.get(&component_id)
    }

    pub(crate) fn interfaces_iter(
        &self,
    ) -> impl Iterator<Item = &Box<ComponentDispatcherInterface + Send + Sync>> {
        self.interfaces.values()
    }
}

struct ComponentDispatcher<T: 'static + SpatialComponent + Sync + Send + Clone + Debug> {
    _phantom: PhantomData<T>,
}

pub(crate) trait ComponentDispatcherInterface {
    fn add_component<'b>(
        &self,
        res: &Resources,
        entity: SpatialEntity,
        add_component: AddComponentOp,
    );
    fn remove_component<'b>(&self, res: &Resources, entity: SpatialEntity);
    fn apply_component_update<'b>(
        &self,
        res: &Resources,
        entity: SpatialEntity,
        component_update: ComponentUpdateOp,
    );
    fn apply_authority_change<'b>(
        &self,
        res: &Resources,
        entity: SpatialEntity,
        authority_change: AuthorityChangeOp,
    );
    fn on_command_request<'b>(
        &self,
        res: &Resources,
        entity: SpatialEntity,
        command_request: CommandRequestOp,
    );
    fn on_command_response<'b>(&self, res: &Resources, command_response: CommandResponseOp);
    fn replicate(&self, res: &Resources, connection: &mut WorkerConnection);
}

impl<T: 'static + SpatialComponent + Sync + Send + Clone + Debug> ComponentDispatcherInterface
    for ComponentDispatcher<T>
{
    fn add_component<'b>(
        &self,
        res: &Resources,
        entity: SpatialEntity,
        add_component: AddComponentOp,
    ) {
        let mut storage: SpatialWriteStorage<T> = SpatialStorage::fetch(res);
        let data = add_component.get::<T>().unwrap().clone();

        storage
            .insert(entity, SynchronisedComponent::new(data))
            .unwrap();
    }

    fn remove_component<'b>(&self, res: &Resources, entity: SpatialEntity) {
        let mut storage: SpatialWriteStorage<T> = SpatialStorage::fetch(res);
        storage.remove(entity);
    }

    fn apply_component_update<'b>(
        &self,
        res: &Resources,
        entity: SpatialEntity,
        component_update: ComponentUpdateOp,
    ) {
        let mut storage: SpatialWriteStorage<T> = SpatialStorage::fetch(res);
        let update = component_update.get::<T>().unwrap().clone();

        storage
            .get_mut(entity)
            .unwrap()
            .apply_update_to_value(update);
    }

    fn apply_authority_change<'b>(
        &self,
        res: &Resources,
        entity: SpatialEntity,
        authority_change: AuthorityChangeOp,
    ) {
        res.fetch_mut::<AuthorityBitSet<T>>()
            .set_authority(entity, authority_change.authority);
    }

    fn on_command_request<'b>(
        &self,
        res: &Resources,
        entity: SpatialEntity,
        command_request: CommandRequestOp,
    ) {
        let mut command_requests = CommandRequests::<T>::fetch(res);
        let request = command_request.get::<T>().unwrap().clone();

        match command_requests.get_mut(entity.into()) {
            Some(requests) => {
                requests.on_request(
                    command_request.request_id,
                    request,
                    command_request.caller_worker_id,
                    command_request.caller_attribute_set,
                );
            }
            None => {
                let mut requests: CommandResponder<T> = Default::default();
                requests.on_request(
                    command_request.request_id,
                    request,
                    command_request.caller_worker_id,
                    command_request.caller_attribute_set,
                );
                command_requests
                    .insert(entity.specs_entity(), requests)
                    .expect("Error inserting new command request object.");
            }
        }
    }

    fn on_command_response<'b>(&self, res: &Resources, command_response: CommandResponseOp) {
        CommandSenderImpl::<T>::got_command_response(res, command_response);
    }

    fn replicate(&self, res: &Resources, connection: &mut WorkerConnection) {
        let entities: ReadStorage<SpatialEntity> = Storage::fetch(res);
        let mut storage: SpatialWriteStorage<T> = SpatialStorage::fetch(res);

        for (entity, component) in (&entities, &mut storage).join() {
            component.replicate(connection, entity.entity_id());
        }

        // Send queued command requests and responses
        CommandSender::<T>::fetch(res).flush_requests(connection);

        let mut responses = CommandRequests::<T>::fetch(res);
        for entity in (&mut responses).join() {
            entity.flush_responses(connection);
        }

        responses.clear_empty_request_objects(res);
    }
}
