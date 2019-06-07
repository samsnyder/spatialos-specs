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
use crate::component_registry::*;

pub struct SpatialReader {
    spatial_to_specs_entity: HashMap<EntityId, Entity>
}

impl SpatialReader {
	pub fn new() -> SpatialReader {
		SpatialReader {
			spatial_to_specs_entity: HashMap::new()
		}
	}

    pub fn setup(res: &mut Resources) {
        WriteStorage::<WrappedEntityId>::setup(res);
    }

	pub fn process(&mut self, res: &Resources) {
        let mut connection = res.fetch_mut::<WorkerConnection>();
		let ops = connection.get_op_list(0);

        for op in &ops {
            match op {
            	WorkerOp::AddEntity(add_entity_op) => {
                    let entity = res.fetch_mut::<EntitiesRes>().create();
                    let mut entity_id_storage = WriteStorage::<WrappedEntityId>::fetch(res);
                    entity_id_storage.insert(entity, WrappedEntityId(add_entity_op.entity_id));

                    self.spatial_to_specs_entity.insert(add_entity_op.entity_id, entity);
                }

            	WorkerOp::AddComponent(add_component) => {
                    match res.fetch::<ComponentRegistry>().get_interface(add_component.component_id) {
                    	None => {},
                    	Some(interface) => {
                    		let entity = self.spatial_to_specs_entity[&add_component.entity_id];
                    		interface.add_component_to_world(res, entity, add_component);
                    	}
                    }
                },
                WorkerOp::ComponentUpdate(update) => {
                    match res.fetch::<ComponentRegistry>().get_interface(update.component_id) {
                        None => {},
                        Some(interface) => {
                            let entity = self.spatial_to_specs_entity[&update.entity_id];
                            interface.apply_component_update(res, entity, update);
                        }
                    }
                },
                _ => {}
            }
        }
	}
}

pub struct SpatialReaderSystemData;

impl<'a> SystemData<'a> for SpatialReaderSystemData {
    fn setup(res: &mut Resources) {
        res.insert(SpatialReader::new());
        SpatialReader::setup(res);
    }

    fn fetch(res: &'a Resources) -> Self {
        res.fetch_mut::<SpatialReader>()
            .process(res);
        SpatialReaderSystemData{}
    }

    fn reads() -> Vec<ResourceId> {
        vec![
            ResourceId::new::<SpatialReader>(),
            ResourceId::new::<WorkerConnection>()
        ]
    }

    // TODO - accurately reflect reads and writes
    fn writes() -> Vec<ResourceId> {
        vec![
            ResourceId::new::<SpatialReader>(),
            ResourceId::new::<WorkerConnection>()
        ]
    }
}

pub struct SpatialReaderSystem;
impl<'a> System<'a> for SpatialReaderSystem {
    type SystemData = SpatialReaderSystemData;

    fn run(&mut self, _: SpatialReaderSystemData) {

    }
}