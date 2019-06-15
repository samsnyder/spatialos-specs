use crate::commands::{
    CommandRequests, CommandRequestsComp, CommandRequestsExt, CommandSender, CommandSenderRes,
};
use crate::entities::SpatialEntity;
use crate::storage::{AuthorityBitSet, SpatialWriteStorage};
use crate::SpatialComponent;
use spatialos_sdk::worker::component::Component as WorkerComponent;
use spatialos_sdk::worker::component::ComponentId;
use spatialos_sdk::worker::connection::WorkerConnection;
use spatialos_sdk::worker::op::{
    AddComponentOp, AuthorityChangeOp, CommandRequestOp, CommandResponseOp, ComponentUpdateOp,
};
use specs::prelude::{Join, ReadStorage, Resources, Storage, SystemData, Write, WriteStorage};
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;

static mut COMPONENT_REGISTRY: Option<ComponentRegistry> = None;

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
    unsafe fn get_registry() -> &'static ComponentRegistry {
        COMPONENT_REGISTRY.get_or_insert_with(|| Default::default())
    }

    unsafe fn get_registry_mut() -> &'static mut ComponentRegistry {
        COMPONENT_REGISTRY.get_or_insert_with(|| Default::default())
    }

    pub(crate) fn register_component<T: 'static + WorkerComponent>() {
        unsafe {
            let interface = ComponentDispatcher::<T> {
                _phantom: PhantomData,
            };
            Self::get_registry_mut()
                .interfaces
                .insert(T::ID, Box::new(interface));
        }
    }

    pub(crate) fn setup_components(res: &mut Resources) {
        unsafe {
            for interface in Self::get_registry().interfaces.values() {
                interface.setup_component(res);
            }
        }
    }

    pub(crate) fn get_interface(
        component_id: ComponentId,
    ) -> Option<&'static Box<ComponentDispatcherInterface + Send + Sync>> {
        unsafe { Self::get_registry().interfaces.get(&component_id) }
    }

    pub(crate) fn interfaces_iter(
    ) -> impl Iterator<Item = &'static Box<ComponentDispatcherInterface + Send + Sync + 'static>>
    {
        unsafe { Self::get_registry().interfaces.values() }
    }
}

struct ComponentDispatcher<T: 'static + WorkerComponent + Sync + Send + Clone + Debug> {
    _phantom: PhantomData<T>,
}

pub(crate) trait ComponentDispatcherInterface {
    fn setup_component(&self, res: &mut Resources);
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

impl<T: 'static + WorkerComponent + Sync + Send + Clone + Debug> ComponentDispatcherInterface
    for ComponentDispatcher<T>
{
    fn setup_component(&self, res: &mut Resources) {
        // Create component data storage.
        WriteStorage::<SpatialComponent<T>>::setup(res);

        // Create command sender resource.
        Write::<CommandSenderRes<T>>::setup(res);

        // Create command receiver storage.
        CommandRequests::<T>::setup(res);

        res.insert(AuthorityBitSet::<T>::new());
    }

    fn add_component<'b>(
        &self,
        res: &Resources,
        entity: SpatialEntity,
        add_component: AddComponentOp,
    ) {
        let mut storage: SpatialWriteStorage<T> = SpatialWriteStorage::fetch(res);
        let data = add_component.get::<T>().unwrap().clone();

        storage
            .insert(entity.specs_entity(), SpatialComponent::new(data))
            .unwrap();
    }

    fn remove_component<'b>(&self, res: &Resources, entity: SpatialEntity) {
        let mut storage: SpatialWriteStorage<T> = SpatialWriteStorage::fetch(res);
        storage.remove(entity.specs_entity());
    }

    fn apply_component_update<'b>(
        &self,
        res: &Resources,
        entity: SpatialEntity,
        component_update: ComponentUpdateOp,
    ) {
        let mut storage: SpatialWriteStorage<T> = SpatialWriteStorage::fetch(res);
        let update = component_update.get::<T>().unwrap().clone();

        storage
            .get_mut(entity.specs_entity())
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
                let mut requests: CommandRequestsComp<T> = Default::default();
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
        CommandSenderRes::<T>::got_command_response(res, command_response);
    }

    fn replicate(&self, res: &Resources, connection: &mut WorkerConnection) {
        let entities: ReadStorage<SpatialEntity> = Storage::fetch(res);
        let mut storage: SpatialWriteStorage<T> = SpatialWriteStorage::fetch(res);

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

// pub struct WriteAndRegisterComponent<'a, T: 'a + Resource, C: WorkerComponent> {
//     resource: Write<'a, T>,
//     phantom: PhantomData<C>,
// }

// impl<'a, T, C: WorkerComponent> Deref for WriteAndRegisterComponent<'a, T, C>
// where
//     T: Resource,
// {
//     type Target = T;

//     fn deref(&self) -> &T {
//         self.resource.deref()
//     }
// }

// impl<'a, T, C: WorkerComponent> DerefMut for WriteAndRegisterComponent<'a, T, C>
// where
//     T: Resource,
// {
//     fn deref_mut(&mut self) -> &mut T {
//         self.resource.deref_mut()
//     }
// }

// impl<'a, T, C: WorkerComponent> SystemData<'a> for WriteAndRegisterComponent<'a, T, C>
// where
//     C: 'static + WorkerComponent,
//     T: Resource + Default,
// {
//     fn setup(res: &mut Resources) {
//         ComponentRegistry::register_component::<C>(res);
//         Write::<T>::setup(res);
//     }

//     fn fetch(res: &'a Resources) -> Self {
//         WriteAndRegisterComponent {
//             resource: Write::fetch(res),
//             phantom: PhantomData,
//         }
//     }

//     fn reads() -> Vec<ResourceId> {
//         Write::<T>::reads()
//     }

//     fn writes() -> Vec<ResourceId> {
//         Write::<T>::writes()
//     }
// }
