use crate::generated::game::*;
use spatialos_sdk::worker::entity::Entity as WorkerEntity;
use spatialos_sdk::worker::entity_builder::EntityBuilder;
use spatialos_specs::*;
use specs::prelude::*;

pub struct ClientBootstrap {
    pub has_requested_player: bool,
}

impl<'a> System<'a> for ClientBootstrap {
    type SystemData = (
        SpatialReadStorage<'a, PlayerCreator>,
        SpatialEntities<'a>,
        CommandSender<'a, PlayerCreator>,
    );

    fn run(&mut self, (creator, entities, mut player_command_sender): Self::SystemData) {
        if !self.has_requested_player {
            match (&creator, &entities).join().next() {
                Some((_, player_creator_entity)) => {
                    self.has_requested_player = true;

                    player_command_sender.send_command(
                        player_creator_entity.entity_id(),
                        PlayerCreatorCommandRequest::CreatePlayer(CreatePlayerRequest {
                            name: "MyName".to_string(),
                        }),
                        |result| {
                            result.get_system_data::<_, Self>(|(_, _entities, _)| {
                                match &*result {
                                    Ok(result) => println!("Created player: {:?}", result),
                                    Err(status) => println!("Error creating player: {:?}", status),
                                }
                            })
                        },
                    )
                }
                None => {}
            }
        }
    }
}

pub struct PlayerCreatorSys;

impl<'a> System<'a> for PlayerCreatorSys {
    type SystemData = (CommandRequests<'a, PlayerCreator>, SystemCommandSender<'a>);

    fn run(&mut self, (mut requests, mut sys_command_sender): Self::SystemData) {
        for request in (&mut requests).join() {
            request.respond(|request, caller_worker_id, _| match request {
                PlayerCreatorCommandRequest::CreatePlayer(request) => {
                    let player_name = request.name.clone();
                    let caller_worker_id = caller_worker_id.clone();

                    sys_command_sender.reserve_entity_ids(1, move |res, result| {
                        let (_, mut sys_command_sender) = <Self as System>::SystemData::fetch(res);

                        let entity = Self::create_player_entity(player_name);

                        let reserved_id = result.unwrap().next().unwrap();

                        sys_command_sender.create_entity(
                            entity,
                            Some(reserved_id),
                            move |_res, result| {
                                println!(
                                    "Created player entity for {}: {:?}",
                                    caller_worker_id, result
                                );
                            },
                        );
                    });

                    Some(PlayerCreatorCommandResponse::CreatePlayer(
                        CreatePlayerResponse {},
                    ))
                }
            });
        }
    }
}

impl PlayerCreatorSys {
    fn create_player_entity(name: String) -> WorkerEntity {
        let mut builder = EntityBuilder::new(0.0, 0.0, 0.0, "managed");

        builder.add_component(
            Player {
                name,
                current_direction: 0,
            },
            "managed",
        );
        builder.set_metadata("Player", "managed");
        builder.set_entity_acl_write_access("managed");

        builder.build().unwrap()
    }
}
