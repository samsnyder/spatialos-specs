#![allow(unused_imports)]
#![allow(unreachable_code)]
#![allow(unreachable_patterns)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(unused_mut)]

use spatialos_sdk::worker::component::*;
use spatialos_sdk::worker::internal::schema::*;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct Coordinates {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone)]
pub struct Position {
    pub coords: Coordinates,
}

impl TypeConversion for Position {
    fn from_type(input: &SchemaObject) -> Result<Self, String> {
        unimplemented!()
    }
    fn to_type(input: &Self, output: &mut SchemaObject) -> Result<(), String> {
        unimplemented!()
    }
}
impl ComponentData<Position> for Position {
    fn merge(&mut self, update: PositionUpdate) {
        unimplemented!()
    }
}

#[derive(Debug, Clone, Default)]
pub struct PositionUpdate {
    pub coords: Option<Coordinates>,
}
impl TypeConversion for PositionUpdate {
    fn from_type(input: &SchemaObject) -> Result<Self, String> {
        unimplemented!()
    }
    fn to_type(input: &Self, output: &mut SchemaObject) -> Result<(), String> {
        unimplemented!()
    }
}
impl ComponentUpdate<Position> for PositionUpdate {
    fn merge(&mut self, update: PositionUpdate) {
        unimplemented!()
    }
}

#[derive(Debug, Clone)]
pub enum PositionCommandRequest {
    UpdateCoords,
}

#[derive(Debug, Clone)]
pub enum PositionCommandResponse {
    UpdateCoords,
}

impl Component for Position {
    type Update = PositionUpdate;
    type CommandRequest = PositionCommandRequest;
    type CommandResponse = PositionCommandResponse;

    const ID: ComponentId = 54;

    fn from_data(data: &SchemaComponentData) -> Result<Position, String> {
        unimplemented!()
    }

    fn from_update(update: &SchemaComponentUpdate) -> Result<PositionUpdate, String> {
        unimplemented!()
    }

    fn from_request(request: &SchemaCommandRequest) -> Result<PositionCommandRequest, String> {
        unimplemented!()
    }

    fn from_response(response: &SchemaCommandResponse) -> Result<PositionCommandResponse, String> {
        unimplemented!()
    }

    fn to_data(data: &Position) -> Result<SchemaComponentData, String> {
        unimplemented!()
    }

    fn to_update(update: &PositionUpdate) -> Result<SchemaComponentUpdate, String> {
        unimplemented!()
    }

    fn to_request(request: &PositionCommandRequest) -> Result<SchemaCommandRequest, String> {
        unimplemented!()
    }

    fn to_response(response: &PositionCommandResponse) -> Result<SchemaCommandResponse, String> {
        unimplemented!()
    }

    fn get_request_command_index(request: &PositionCommandRequest) -> u32 {
        unimplemented!()
    }

    fn get_response_command_index(response: &PositionCommandResponse) -> u32 {
        unimplemented!()
    }
}
