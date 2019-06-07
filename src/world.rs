use spatialos_sdk::worker::component::Component as SpatialComponent;
use specs::prelude::*;
use spatialos_sdk::worker::connection::*;
use spatialos_sdk::worker::op::*;
use std::marker::PhantomData;
use std::collections::HashMap;
use spatialos_sdk::worker::*;
use specs::world::Index;
use spatialos_sdk::worker::component::{UpdateParameters, ComponentId};
use spatialos_sdk::worker::component::VTable;
use std::fmt::Debug;
use crate::*;
use crate::storage::*;

struct ComponentRegistry {
    interfaces: HashMap<ComponentId, Box<ComponentDispatcherInterface + Send + Sync>>
}

impl ComponentRegistry {
    fn new() -> ComponentRegistry {
        ComponentRegistry {
            interfaces: HashMap::new()
        }
    }

    fn register_component<C: 'static + SpatialComponent>(&mut self) {
        let interface = ComponentDispatcher::<C>{
            _phantom: PhantomData
        };
        self.interfaces.insert(C::ID, Box::new(interface));
    }

    fn get_interface(&self, component_id: ComponentId) -> Option<&Box<ComponentDispatcherInterface + Send + Sync>> {
        self.interfaces.get(&component_id)
    }

    fn interfaces_iter(&self) -> impl Iterator<Item = &Box<ComponentDispatcherInterface + Send + Sync>> {
        self.interfaces.values()
    }
}

struct ComponentDispatcher<C: 'static + SpatialComponent + Sync + Send + Clone + Debug> {
	_phantom: PhantomData<C>
}

trait ComponentDispatcherInterface {
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

pub struct WorldReader {
	spatial_to_specs_entity: HashMap<EntityId, Entity>,
    connection: WorkerConnection
}

impl WorldReader {
	pub fn new(connection: WorkerConnection) -> WorldReader {
		WorldReader {
			spatial_to_specs_entity: HashMap::new(),
            connection
		}
	}

    pub fn setup(&self, world: &mut World) {
        world.register::<WrappedEntityId>();
    }

    pub fn register_component<C: 'static + SpatialComponent>(res: &mut Resources) {
        res.entry::<ComponentRegistry>()
            .or_insert_with(|| ComponentRegistry::new())
            .register_component::<C>();
    }

	pub fn process(&mut self, world: &mut World) {
		let ops = self.connection.get_op_list(0);

        for op in &ops {
            match op {
            	WorkerOp::AddEntity(add_entity_op) => {
                    let entity = world.create_entity().with(WrappedEntityId(add_entity_op.entity_id)).build();
                    self.spatial_to_specs_entity.insert(add_entity_op.entity_id, entity);
                }

            	WorkerOp::AddComponent(add_component) => {
                    match world.res.fetch::<ComponentRegistry>().get_interface(add_component.component_id) {
                    	None => {},
                    	Some(interface) => {
                    		let entity = self.spatial_to_specs_entity[&add_component.entity_id];
                    		interface.add_component_to_world(&world.res, entity, add_component);
                    	}
                    }
                },
                WorkerOp::ComponentUpdate(update) => {
                    match world.res.fetch::<ComponentRegistry>().get_interface(update.component_id) {
                        None => {},
                        Some(interface) => {
                            let entity = self.spatial_to_specs_entity[&update.entity_id];
                            interface.apply_component_update(&world.res, entity, update);
                        }
                    }
                },
                _ => {}
            }
        }
	}

    pub fn replicate(&mut self, res: &Resources) {
        for interface in res.fetch::<ComponentRegistry>().interfaces_iter() {
            interface.replicate(&res, &mut self.connection);
        }
    }
}