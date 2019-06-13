extern crate spatialos_specs;
extern crate specs;

use crate::{connection_handler::*, opt::*};
use generated::{game, improbable};
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

use crate::generated::game::*;
use crate::generated::improbable::*;
use spatialos_specs::commands::*;
use spatialos_specs::entities::*;
use spatialos_specs::system_commands::*;
use spatialos_specs::*;

use crate::player_connection::*;
use std::thread;
use std::time::Duration;

pub mod connection_handler;
#[rustfmt::skip]
pub mod generated;
pub mod opt;
pub mod player_connection;
pub mod player;

pub use self::connection_handler::*;
pub use self::opt::*;
