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

    fn run(&mut self, (_, entities, mut player_command_sender): Self::SystemData) {
        if (!self.has_requested_player) {
            match entities.join().next() {
                Some(player_creator_entity) => {
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

            // player_command_sender.send_command()
        }

        // for (entity, pos) in (&entities, &mut pos).join() {
        //     println!("write: {:?}", pos.coords);
        //     pos.coords.x = rng.gen();

        //     let this_entity = *entity;

        //     example_command_sender.send_command(
        //         this_entity.entity_id(),
        //         ExampleCommandRequest::TestCommand(CommandData { value: rng.gen() }),
        //         move |res, response| {

        //             let mut storage = SpatialWriteStorage::<Position>::fetch(res);
        //             storage.get_mut(this_entity).unwrap().coords.x = 5.0;

        //             match response {
        //                 Ok(response) => println!("response {:?}", response),
        //                 Err(err) => println!("error {:?}", err)
        //             };

        //             let player_entity = create_player_entity(true);

        //             SystemCommandSender::fetch(res).create_entity(player_entity, |res, entity_id| {
        //                 println!("created entity! {:?}", entity_id);
        //                 let sender = SystemCommandSender::fetch(res);
        //             });
        //         }
        //     );
        // }
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
