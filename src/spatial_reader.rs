use crate::component_registry::ComponentRegistry;
use crate::entities::{SpatialEntities, SpatialEntitiesWrite};
use crate::system_commands::{SystemCommandSender, SystemCommandSenderRes};
use spatialos_sdk::worker::connection::{Connection, WorkerConnection};
use spatialos_sdk::worker::op::WorkerOp;
use specs::prelude::{Resources, System, SystemData};
use specs::shred::ResourceId;
use specs::world::EntitiesRes;

#[doc(hidden)]
pub struct ResourcesSystemData<'a> {
    pub(crate) res: &'a Resources,
}

impl<'a> SystemData<'a> for ResourcesSystemData<'a> {
    fn setup(_: &mut Resources) {}

    fn fetch(res: &'a Resources) -> Self {
        ResourcesSystemData { res }
    }

    fn reads() -> Vec<ResourceId> {
        vec![]
    }

    fn writes() -> Vec<ResourceId> {
        vec![
            ResourceId::new::<EntitiesRes>(),
            ResourceId::new::<WorkerConnection>(),
        ]
    }
}

pub struct SpatialReaderSystem;

impl<'a> System<'a> for SpatialReaderSystem {
    type SystemData = ResourcesSystemData<'a>;

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);

        SystemCommandSender::setup(res);
        SpatialEntitiesWrite::setup(res);
    }

    fn run(&mut self, res: Self::SystemData) {
        let res = res.res;

        let ops = {
            let mut connection = res.fetch_mut::<WorkerConnection>();
            connection.get_op_list(0)
        };

        for op in &ops {
            match op {
                WorkerOp::AddEntity(add_entity_op) => {
                    SpatialEntitiesWrite::fetch(res).got_new_entity(res, add_entity_op.entity_id);
                }
                WorkerOp::RemoveEntity(remove_entity_op) => {
                    SpatialEntitiesWrite::fetch(res).remove_entity(res, remove_entity_op.entity_id);
                }
                WorkerOp::AddComponent(add_component) => {
                    match res
                        .fetch::<ComponentRegistry>()
                        .get_interface(add_component.component_id)
                    {
                        None => {}
                        Some(interface) => {
                            let entity = SpatialEntities::fetch(res)
                                .get_entity(add_component.entity_id)
                                .unwrap();
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
                            let entity = SpatialEntities::fetch(res)
                                .get_entity(remove_component.entity_id)
                                .unwrap();
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
                            let entity = SpatialEntities::fetch(res)
                                .get_entity(update.entity_id)
                                .unwrap();
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
                            let entity = SpatialEntities::fetch(res)
                                .get_entity(authority_change.entity_id)
                                .unwrap();
                            interface.apply_authority_change(res, entity, authority_change);
                        }
                    }
                }
                WorkerOp::CommandRequest(command_request) => {
                    match res
                        .fetch::<ComponentRegistry>()
                        .get_interface(command_request.component_id)
                    {
                        None => {}
                        Some(interface) => {
                            let entity = SpatialEntities::fetch(res)
                                .get_entity(command_request.entity_id)
                                .unwrap();
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
                            interface.on_command_response(res, command_response);
                        }
                    }
                }
                WorkerOp::ReserveEntityIdsResponse(reserve_entity_ids_response) => {
                    SystemCommandSenderRes::got_reserve_entity_ids_response(
                        res,
                        reserve_entity_ids_response,
                    );
                }
                WorkerOp::CreateEntityResponse(create_entity_response) => {
                    SystemCommandSenderRes::got_create_entity_response(res, create_entity_response);
                }
                WorkerOp::DeleteEntityResponse(delete_entity_response) => {
                    SystemCommandSenderRes::got_delete_entity_response(res, delete_entity_response);
                }
                WorkerOp::EntityQueryResponse(entity_query_response) => {
                    SystemCommandSenderRes::got_entity_query_response(res, entity_query_response);
                }
                _ => {}
            }
        }
    }
}
