extern crate structopt;

use example::player::*;
use example::player_connection::*;
use example::{connection_handler::*, opt::*};
use spatialos_sdk::worker::connection::Connection;
use spatialos_specs::*;
use specs::prelude::*;
use std::thread;
use std::time::Duration;
use structopt::StructOpt;

fn main() {
    let opt = Opt::from_args();
    let connection = match get_connection(opt) {
        Ok(c) => c,
        Err(e) => panic!("{}", e),
    };

    println!("Connected as: {}", connection.get_worker_id());

    let mut world = World::new();

    world.add_resource(connection);

    let mut dispatcher = DispatcherBuilder::new()
        .with(SpatialReaderSystem, "reader", &[])
        .with_barrier()
        .with(
            ClientBootstrap {
                has_requested_player: false,
            },
            "",
            &[],
        )
        .with(PlayerCreatorSys, "", &[])
        .with(MovePlayerSys, "", &[])
        .with_barrier()
        .with(SpatialWriterSystem, "writer", &[])
        .build();

    dispatcher.setup(&mut world.res);

    loop {
        dispatcher.dispatch(&world.res);

        thread::sleep(Duration::from_millis(30));
    }
}
