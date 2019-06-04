use spatialos_sdk::worker::component::Component as SpatialComponent;
use specs::prelude::*;
use spatialos_sdk::worker::connection::*;
use spatialos_sdk::worker::op::*;
use std::marker::PhantomData;
use std::collections::HashMap;
use spatialos_sdk::worker::*;
use specs::world::Index;
use spatialos_sdk::worker::component::ComponentId;
use spatialos_sdk::worker::component::VTable;
use crate::*;

struct ComponentDispatcher<C: 'static + SpatialComponent + Sync + Send + Clone> {
	_phantom: PhantomData<C>
}

trait ComponentDispatcherInterface {
	fn add_component_to_world<'b>(&self, world: &mut World, entity: Entity, add_component: AddComponentOp);
}

impl<T: 'static + SpatialComponent + Sync + Send + Clone> ComponentDispatcherInterface for ComponentDispatcher<T> {
	fn add_component_to_world<'b>(&self, world: &mut World, entity: Entity, add_component: AddComponentOp) {
		let mut storage = world.system_data::<WriteStorage<SynchronisedComponent<T>>>();
		let data: T = add_component.get::<T>().unwrap().clone();

		storage.insert(entity, SynchronisedComponent::new(data));
	}
}

pub struct WorldReader {
	interfaces: HashMap<ComponentId, Box<ComponentDispatcherInterface>>,
	spatial_to_specs_entity: HashMap<EntityId, Entity>
}

impl WorldReader {
	pub fn new() -> WorldReader {
		let reader = WorldReader {
			interfaces: HashMap::new(),
			spatial_to_specs_entity: HashMap::new()
		};

		// for (vtable) in inventory::iter::<VTable>.into_iter() {
            
  //       }

		reader
	}

    pub fn register_component<C: 'static + SpatialComponent + Sync + Send + Clone>(&mut self) {
        let interface = ComponentDispatcher::<C>{
            _phantom: PhantomData
        };
        self.interfaces.insert(C::ID, Box::new(interface));
    }

	pub fn process(&mut self, connection: &mut WorkerConnection, world: &mut World) {
		let ops = connection.get_op_list(0);

        // Process ops.
        for op in &ops {
            if let WorkerOp::Metrics(_) = op {
                println!("Received metrics.");
            } else {
                println!("Received op: {:?}", op);
            }

            match op {
            	WorkerOp::AddEntity(add_entity_op) => {
                    let entity = world.create_entity().build();
                    self.spatial_to_specs_entity.insert(add_entity_op.entity_id, entity);
                }

            	WorkerOp::AddComponent(add_component) => {
                    match self.interfaces.get_mut(&add_component.component_id) {
                    	None => println!("Unknown component: {:?}", add_component.component_id),
                    	Some(interface) => {
                    		let entity = self.spatial_to_specs_entity[&add_component.entity_id];

                    		interface.add_component_to_world(world, entity, add_component);
                    	}
                    }
                },
                _ => {}
            }
        }
	}
}