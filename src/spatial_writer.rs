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

pub struct SpatialWriter;

impl SpatialWriter {
	pub fn new() -> SpatialWriter {
		SpatialWriter {}
	}

    pub fn process(&mut self, res: &Resources) {
    	let mut connection = res.fetch_mut::<WorkerConnection>();

        for interface in res.fetch::<ComponentRegistry>().interfaces_iter() {
            interface.replicate(&res, &mut connection);
        }
    }
}

pub struct SpatialWriterSystemData;

impl<'a> SystemData<'a> for SpatialWriterSystemData {
    fn setup(res: &mut Resources) {
    	res.insert(SpatialWriter::new());
    }

    fn fetch(res: &'a Resources) -> Self {
        res.fetch_mut::<SpatialWriter>()
            .process(res);
        SpatialWriterSystemData{}
    }

    fn reads() -> Vec<ResourceId> {
        vec![
            ResourceId::new::<SpatialWriter>(),
            ResourceId::new::<WorkerConnection>()
        ]
    }

    // TODO - accurately reflect reads and writes
    fn writes() -> Vec<ResourceId> {
        vec![
            ResourceId::new::<SpatialWriter>(),
            ResourceId::new::<WorkerConnection>()
        ]
    }
}

pub struct SpatialWriterSystem;
impl<'a> System<'a> for SpatialWriterSystem {
    type SystemData = SpatialWriterSystemData;

    fn run(&mut self, _: SpatialWriterSystemData) {

    }
}