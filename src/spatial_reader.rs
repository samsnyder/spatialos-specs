use crate::component_registry::*;
use crate::system_commands::*;
use crate::storage::*;
use crate::entities::*;
use crate::*;
use spatialos_sdk::worker::component::Component as SpatialComponent;
use spatialos_sdk::worker::component::VTable;
use spatialos_sdk::worker::component::{ComponentId, UpdateParameters};
use spatialos_sdk::worker::connection::*;
use spatialos_sdk::worker::op::*;
use specs::prelude::*;
use specs::world::*;
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;

pub struct SpatialReader {
}

impl SpatialReader {
    pub fn new() -> SpatialReader {
        SpatialReader {
        }
    }

    pub fn setup(res: &mut Resources) {
        SystemCommandSender::setup(res);
        SpatialEntitiesWrite::setup(res);
    }

    pub fn process(&mut self, res: &Resources) {
        let mut connection = res.fetch_mut::<WorkerConnection>();
        let ops = connection.get_op_list(0);

        for op in &ops {
            match op {
                WorkerOp::AddEntity(add_entity_op) => {
                    SpatialEntitiesWrite::fetch(res).got_new_entity(res, add_entity_op.entity_id);
                }
                WorkerOp::RemoveEntity(remove_entity_op) => {
                    SpatialEntitiesWrite::fetch(res).remove_entity(remove_entity_op.entity_id);
                }
                WorkerOp::AddComponent(add_component) => {
                    match res
                        .fetch::<ComponentRegistry>()
                        .get_interface(add_component.component_id)
                    {
                        None => {}
                        Some(interface) => {
                            let entity = SpatialEntities::fetch(res).get_entity(add_component.entity_id).unwrap();
                            interface.add_component(res, entity, add_component);
                        }
                    }
                }
                WorkerOp::RemoveComponent(remove_component) => {
                    match res
                        .fetch::<ComponentRegistry>()
                        .get_interface(remove_component.component_id)
                    {
                        None => {}
                        Some(interface) => {
                            let entity = SpatialEntities::fetch(res).get_entity(remove_component.entity_id).unwrap();
                            interface.remove_component(res, entity);
                        }
                    }
                }
                WorkerOp::ComponentUpdate(update) => {
                    match res
                        .fetch::<ComponentRegistry>()
                        .get_interface(update.component_id)
                    {
                        None => {}
                        Some(interface) => {
                            let entity = SpatialEntities::fetch(res).get_entity(update.entity_id).unwrap();
                            interface.apply_component_update(res, entity, update);
                        }
                    }
                }
                WorkerOp::AuthorityChange(authority_change) => {
                    match res
                        .fetch::<ComponentRegistry>()
                        .get_interface(authority_change.component_id)
                    {
                        None => {}
                        Some(interface) => {
                            let entity = SpatialEntities::fetch(res).get_entity(authority_change.entity_id).unwrap();
                            interface.apply_authority_change(res, entity, authority_change);
                        }
                    }
                },
                WorkerOp::CommandRequest(command_request) => {
                    match res
                        .fetch::<ComponentRegistry>()
                        .get_interface(command_request.component_id)
                    {
                        None => {}
                        Some(interface) => {
                            let entity = SpatialEntities::fetch(res).get_entity(command_request.entity_id).unwrap();
                            interface.on_command_request(res, entity, command_request);
                        }
                    }
                }
                WorkerOp::CommandResponse(command_response) => {
                    match res
                        .fetch::<ComponentRegistry>()
                        .get_interface(command_response.component_id)
                    {
                        None => {}
                        Some(interface) => {
                            let entity = SpatialEntities::fetch(res).get_entity(command_response.entity_id).unwrap();
                            interface.on_command_response(res, entity, command_response);
                        }
                    }
                }
                WorkerOp::CreateEntityResponse(create_entity_response) => {
                    SystemCommandSenderImpl::got_create_entity_response(res, create_entity_response);
                }
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
        res.fetch_mut::<SpatialReader>().process(res);
        SpatialReaderSystemData {}
    }

    fn reads() -> Vec<ResourceId> {
        vec![]
    }

    // TODO - accurately reflect reads and writes
    fn writes() -> Vec<ResourceId> {
        vec![
            ResourceId::new::<EntitiesRes>(),
            ResourceId::new::<SpatialReader>(),
            ResourceId::new::<WorkerConnection>(),
        ]
    }
}

pub struct SpatialReaderSystem;
impl<'a> System<'a> for SpatialReaderSystem {
    type SystemData = SpatialReaderSystemData;

    fn run(&mut self, _: SpatialReaderSystemData) {}
}
