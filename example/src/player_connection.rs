use specs::prelude::*;

use spatialos_sdk::worker::connection::WorkerConnection;
use spatialos_sdk::worker::entity::Entity as WorkerEntity;
use spatialos_sdk::worker::entity_builder::EntityBuilder;
use spatialos_specs::spatial_reader::*;
use spatialos_specs::spatial_writer::*;
use spatialos_specs::storage::*;

use crate::generated::game::*;
use crate::generated::improbable::*;
use spatialos_specs::commands::*;
use spatialos_specs::entities::*;
use spatialos_specs::system_commands::*;
use spatialos_specs::*;

use std::thread;
use std::time::Duration;

use rand::Rng;

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
        if (!self.has_requested_player) {
            match (&creator, &entities).join().next() {
                Some((_, player_creator_entity)) => {
                    self.has_requested_player = true;

                    player_command_sender.send_command(
                        player_creator_entity.entity_id(),
                        PlayerCreatorCommandRequest::CreatePlayer(CreatePlayerRequest {
                            name: "MyName".to_string(),
                        }),
                        |res, result| {
                            println!("Created player! {:?}", result);
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
            request.respond(|request| match request {
                PlayerCreatorCommandRequest::CreatePlayer(request) => {
                    let entity = Self::create_player_entity(request.name.clone());

                    sys_command_sender.create_entity(entity, |_res, result| {
                        println!("Created player entity: {:?}", result);
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

        builder.add_component(Player { name }, "managed");
        builder.set_metadata("Player", "managed");
        builder.set_entity_acl_write_access("managed");

        builder.build().unwrap()
    }
}
