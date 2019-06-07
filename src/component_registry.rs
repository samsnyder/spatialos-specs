use spatialos_sdk::worker::component::Component as SpatialComponent;
use specs::prelude::*;
use spatialos_sdk::worker::connection::*;
use spatialos_sdk::worker::op::*;
use std::marker::PhantomData;
use std::collections::HashMap;
use spatialos_sdk::worker::*;
use specs::world::*;
use spatialos_sdk::worker::component::{UpdateParameters, ComponentId};
use spatialos_sdk::worker::component::VTable;
use std::fmt::Debug;
use crate::*;
use crate::storage::*;


pub(crate) struct ComponentRegistry {
    interfaces: HashMap<ComponentId, Box<ComponentDispatcherInterface + Send + Sync>>
}

impl ComponentRegistry {
    fn new() -> ComponentRegistry {
        ComponentRegistry {
            interfaces: HashMap::new()
        }
    }

    pub(crate) fn register_component<C: 'static + SpatialComponent>(res: &mut Resources) {
        res.entry::<ComponentRegistry>()
            .or_insert_with(|| ComponentRegistry::new())
            .register_component_on_self::<C>();
    }

    fn register_component_on_self<C: 'static + SpatialComponent>(&mut self) {
        let interface = ComponentDispatcher::<C>{
            _phantom: PhantomData
        };
        self.interfaces.insert(C::ID, Box::new(interface));
    }

    pub(crate) fn get_interface(&self, component_id: ComponentId) -> Option<&Box<ComponentDispatcherInterface + Send + Sync>> {
        self.interfaces.get(&component_id)
    }

    pub(crate) fn interfaces_iter(&self) -> impl Iterator<Item = &Box<ComponentDispatcherInterface + Send + Sync>> {
        self.interfaces.values()
    }
}

struct ComponentDispatcher<C: 'static + SpatialComponent + Sync + Send + Clone + Debug> {
	_phantom: PhantomData<C>
}

pub(crate) trait ComponentDispatcherInterface {
	fn add_component_to_world<'b>(&self, res: &Resources, entity: Entity, add_component: AddComponentOp);
    fn apply_component_update<'b>(&self, res: &Resources, entity: Entity, component_update: ComponentUpdateOp);
    fn replicate(&self, res: &Resources, connection: &mut WorkerConnection);
}

impl<T: 'static + SpatialComponent + Sync + Send + Clone + Debug> ComponentDispatcherInterface for ComponentDispatcher<T> {
	fn add_component_to_world<'b>(&self, res: &Resources, entity: Entity, add_component: AddComponentOp) {
		let mut storage: SpatialWriteStorage<T> = SpatialStorage::fetch(res);
		let data = add_component.get::<T>().unwrap().clone();

		storage.insert(entity, SynchronisedComponent::new(data));
	}

    fn apply_component_update<'b>(&self, res: &Resources, entity: Entity, component_update: ComponentUpdateOp) {
        let mut storage: SpatialWriteStorage<T> = SpatialStorage::fetch(res);
        let update = component_update.get::<T>().unwrap().clone();

        storage.get_mut(entity).unwrap().apply_update(update);
    }

    fn replicate(&self, res: &Resources, connection: &mut WorkerConnection) {
        let entity_ids: ReadStorage<WrappedEntityId> = Storage::fetch(res);
        let mut storage: SpatialWriteStorage<T> = SpatialStorage::fetch(res);

        for (entity_id, component) in (&entity_ids, &mut storage).join() {
            if component.get_and_clear_dity_bit() {
                connection.send_component_update::<T>(
                    entity_id.0,
                    component.to_update(),
                    UpdateParameters::default(),
                );
            }
        }
    }
}
