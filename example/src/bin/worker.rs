extern crate structopt;

use example::generated::{game, improbable};
use example::{connection_handler::*, opt::*};
use rand::Rng;
use spatialos_sdk::worker::{
    commands::{EntityQueryRequest, ReserveEntityIdsRequest},
    component::{Component, ComponentData, UpdateParameters},
    connection::{Connection, WorkerConnection},
    entity_builder::EntityBuilder,
    metrics::{HistogramMetric, Metrics},
    op::{StatusCode, WorkerOp},
    query::{EntityQuery, QueryConstraint, ResultType},
    {EntityId, InterestOverride, LogLevel},
};
use specs::prelude::*;
use std::{collections::HashMap, f64};
use structopt::StructOpt;

use spatialos_sdk::worker::entity::Entity as WorkerEntity;
use spatialos_specs::spatial_reader::*;
use spatialos_specs::spatial_writer::*;
use spatialos_specs::storage::*;

use example::generated::game::*;
use example::generated::improbable::*;
use spatialos_specs::commands::*;
use spatialos_specs::entities::*;
use spatialos_specs::system_commands::*;
use spatialos_specs::*;

use example::get_connection;
use example::player_connection::*;
use std::thread;
use std::time::Duration;

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
        .with_barrier()
        .with(SpatialWriterSystem, "writer", &[])
        .build();

    dispatcher.setup(&mut world.res);

    loop {
        dispatcher.dispatch(&world.res);

        thread::sleep(Duration::from_millis(30));
    }
}
