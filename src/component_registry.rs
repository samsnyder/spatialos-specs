use crate::commands::{
    CommandRequests, CommandRequestsComp, CommandRequestsExt, CommandSender, CommandSenderRes,
};
use crate::entities::EntityIds;
use crate::storage::{AuthorityBitSet, SpatialWriteStorage};
use crate::SpatialComponent;
use spatialos_sdk::worker::component::Component as WorkerComponent;
use spatialos_sdk::worker::component::ComponentId;
use spatialos_sdk::worker::connection::WorkerConnection;
use spatialos_sdk::worker::op::{
    AddComponentOp, AuthorityChangeOp, CommandRequestOp, CommandResponseOp, ComponentUpdateOp,
};
use specs::prelude::{Entity, Join, Resources, SystemData};
use specs::storage::MaskedStorage;
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Mutex;

lazy_static! {
    static ref COMPONENT_REGISTRY: Mutex<ComponentRegistry> = Mutex::new(Default::default());
}

pub(crate) struct ComponentRegistry {
    interfaces: HashMap<ComponentId, Box<ComponentDispatcherInterface + Send + Sync>>,
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        ComponentRegistry {
            interfaces: HashMap::new(),
        }
    }
}

impl ComponentRegistry {
    // SAFETY - The registry is only retrieved after all components have
    // been registered.
    //
    // Therefore, all mutations to the registry happen before this is ever called.
    fn get_registry() -> &'static ComponentRegistry {
        let registry_ptr = &(*COMPONENT_REGISTRY.lock().unwrap()) as *const ComponentRegistry;
        unsafe { &(*registry_ptr) }
    }

    pub(crate) fn register_component<T: 'static + WorkerComponent>() {
        let interface = ComponentDispatcher::<T> {
            _phantom: PhantomData,
        };
        COMPONENT_REGISTRY
            .lock()
            .unwrap()
            .interfaces
            .insert(T::ID, Box::new(interface));
    }

    pub(crate) fn get_interface(
        component_id: ComponentId,
    ) -> Option<&'static Box<ComponentDispatcherInterface + Send + Sync>> {
        Self::get_registry().interfaces.get(&component_id)
    }

    pub(crate) fn interfaces_iter(
    ) -> impl Iterator<Item = &'static Box<ComponentDispatcherInterface + Send + Sync + 'static>>
    {
        ComponentRegistry::get_registry().interfaces.values()
    }
}

#[derive(Clone)]
struct ComponentDispatcher<T: 'static + WorkerComponent + Sync + Send + Clone + Debug> {
    _phantom: PhantomData<T>,
}

pub(crate) trait ComponentDispatcherInterface {
    fn add_component<'b>(&self, res: &Resources, entity: Entity, add_component: AddComponentOp);
    fn remove_component<'b>(&self, res: &Resources, entity: Entity);
    fn apply_component_update<'b>(
        &self,
        res: &Resources,
        entity: Entity,
        component_update: ComponentUpdateOp,
    );
    fn apply_authority_change<'b>(
        &self,
        res: &Resources,
        entity: Entity,
        authority_change: AuthorityChangeOp,
    );
    fn on_command_request<'b>(
        &self,
        res: &Resources,
        entity: Entity,
        command_request: CommandRequestOp,
    );
    fn on_command_response<'b>(&self, res: &Resources, command_response: CommandResponseOp);
    fn replicate(&self, res: &Resources, connection: &mut WorkerConnection);
}

impl<T: 'static + WorkerComponent + Sync + Send + Clone + Debug> ComponentDispatcherInterface
    for ComponentDispatcher<T>
{
    fn add_component<'b>(&self, res: &Resources, entity: Entity, add_component: AddComponentOp) {
        if let Some(mut storage) = SpatialWriteStorage::<T>::try_fetch_component_storage(res) {
            let data = add_component.get::<T>().unwrap().clone();

            storage.insert(entity, SpatialComponent::new(data)).unwrap();
        }
    }

    fn remove_component<'b>(&self, res: &Resources, entity: Entity) {
        if let Some(mut storage) = SpatialWriteStorage::<T>::try_fetch_component_storage(res) {
            storage.remove(entity);
        }
    }

    fn apply_component_update<'b>(
        &self,
        res: &Resources,
        entity: Entity,
        component_update: ComponentUpdateOp,
    ) {
        if let Some(mut storage) = SpatialWriteStorage::<T>::try_fetch_component_storage(res) {
            let update = component_update.get::<T>().unwrap().clone();

            storage
                .get_mut(entity)
                .unwrap()
                .apply_update_to_value(update);
        }
    }

    fn apply_authority_change<'b>(
        &self,
        res: &Resources,
        entity: Entity,
        authority_change: AuthorityChangeOp,
    ) {
        if res.has_value::<AuthorityBitSet<T>>() {
            res.fetch_mut::<AuthorityBitSet<T>>()
                .set_authority(entity, authority_change.authority);
        }
    }

    fn on_command_request<'b>(
        &self,
        res: &Resources,
        entity: Entity,
        command_request: CommandRequestOp,
    ) {
        if res.has_value::<MaskedStorage<CommandRequestsComp<T>>>() {
            let mut command_requests = CommandRequests::<T>::fetch(res);
            let request = command_request.get::<T>().unwrap().clone();

            match command_requests.get_mut(entity) {
                Some(requests) => {
                    requests.on_request(
                        command_request.request_id,
                        request,
                        command_request.caller_worker_id,
                        command_request.caller_attribute_set,
                    );
                }
                None => {
                    let mut requests: CommandRequestsComp<T> = Default::default();
                    requests.on_request(
                        command_request.request_id,
                        request,
                        command_request.caller_worker_id,
                        command_request.caller_attribute_set,
                    );
                    command_requests
                        .insert(entity, requests)
                        .expect("Error inserting new command request object.");
                }
            }
        }
    }

    fn on_command_response<'b>(&self, res: &Resources, command_response: CommandResponseOp) {
        if res.has_value::<CommandSenderRes<T>>() {
            CommandSenderRes::<T>::got_command_response(res, command_response);
        }
    }

    fn replicate(&self, res: &Resources, connection: &mut WorkerConnection) {
        if let Some(mut storage) = SpatialWriteStorage::<T>::try_fetch_component_storage(res) {
            let entity_ids = EntityIds::fetch(res);

            for (entity_id, component) in (&entity_ids, &mut storage).join() {
                component.replicate(connection, *entity_id);
            }
        }

        if res.has_value::<CommandSenderRes<T>>() {
            CommandSender::<T>::fetch(res).flush_requests(connection);
        }

        if res.has_value::<MaskedStorage<CommandRequestsComp<T>>>() {
            let mut responses = CommandRequests::<T>::fetch(res);
            for entity in (&mut responses).join() {
                entity.flush_responses(connection);
            }

            responses.clear_empty_request_objects(res);
        }
    }
}
