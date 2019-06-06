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

struct ComponentDispatcher<C: 'static + SpatialComponent + Sync + Send + Clone + Debug> {
	_phantom: PhantomData<C>
}

trait ComponentDispatcherInterface {
	fn add_component_to_world<'b>(&self, res: &mut Resources, entity: Entity, add_component: AddComponentOp);
    fn apply_component_update<'b>(&self, res: &mut Resources, entity: Entity, component_update: ComponentUpdateOp);
    fn replicate(&self, res: &mut Resources, connection: &mut WorkerConnection);
}

impl<T: 'static + SpatialComponent + Sync + Send + Clone + Debug> ComponentDispatcherInterface for ComponentDispatcher<T> {
	fn add_component_to_world<'b>(&self, res: &mut Resources, entity: Entity, add_component: AddComponentOp) {
		let mut storage = world.system_data::<WriteStorage<SynchronisedComponent<T>>>();
		let data = add_component.get::<T>().unwrap().clone();

		storage.insert(entity, SynchronisedComponent::new(data));
	}

    fn apply_component_update<'b>(&self, res: &mut Resources, entity: Entity, component_update: ComponentUpdateOp) {
        let mut storage = world.system_data::<WriteStorage<SynchronisedComponent<T>>>();
        let update = component_update.get::<T>().unwrap().clone();

        storage.get_mut(entity).unwrap().apply_update(update);
    }

    fn replicate(&self, res: &mut Resources, connection: &mut WorkerConnection) {
        let entity_ids = world.system_data::<ReadStorage<WrappedEntityId>>();
        let mut storage = world.system_data::<WriteStorage<SynchronisedComponent<T>>>();

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
	interfaces: HashMap<ComponentId, Box<ComponentDispatcherInterface>>,
	spatial_to_specs_entity: HashMap<EntityId, Entity>,
    connection: WorkerConnection
}

impl WorldReader {
	pub fn new(connection: WorkerConnection) -> WorldReader {
		let reader = WorldReader {
			interfaces: HashMap::new(),
			spatial_to_specs_entity: HashMap::new()
		};

		// for (vtable) in inventory::iter::<VTable>.into_iter() {
            
  //       }

		reader
	}

    pub fn setup(&self, world: &mut World) {
        world.register::<WrappedEntityId>();
    }

    pub fn register_component<C: 'static + SpatialComponent + Sync + Send + Clone + Debug>(&mut self) {
        let interface = ComponentDispatcher::<C>{
            _phantom: PhantomData
        };
        self.interfaces.insert(C::ID, Box::new(interface));
    }

	pub fn process(&mut self, connection: &mut WorkerConnection, res: &mut Resources) {
		let ops = connection.get_op_list(0);

        for op in &ops {
            match op {
            	WorkerOp::AddEntity(add_entity_op) => {
                    let entity = world.create_entity().with(WrappedEntityId(add_entity_op.entity_id)).build();
                    self.spatial_to_specs_entity.insert(add_entity_op.entity_id, entity);
                }

            	WorkerOp::AddComponent(add_component) => {
                    match self.interfaces.get_mut(&add_component.component_id) {
                    	None => {},
                    	Some(interface) => {
                    		let entity = self.spatial_to_specs_entity[&add_component.entity_id];
                    		interface.add_component_to_world(world, entity, add_component);
                    	}
                    }
                },
                WorkerOp::ComponentUpdate(update) => {
                    match self.interfaces.get_mut(&update.component_id) {
                        None => {},
                        Some(interface) => {
                            let entity = self.spatial_to_specs_entity[&update.entity_id];
                            interface.apply_component_update(world, entity, update);
                        }
                    }
                },
                _ => {}
            }
        }
	}

    pub fn replicate(&mut self, connection: &mut WorkerConnection, world: &mut World) {
        for interface in self.interfaces.values() {
            interface.replicate(world, connection);
        }
    }
}