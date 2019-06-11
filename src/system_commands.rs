use spatialos_sdk::worker::component::Component as SpatialComponent;
use spatialos_sdk::worker::component::TypeConversion;
use spatialos_sdk::worker::internal::schema::SchemaComponentUpdate;
use spatialos_sdk::worker::op::*;
use specs::prelude::*;
use specs::shred::{Fetch, Resource, ResourceId, SystemData};
use specs::storage::MaskedStorage;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use spatialos_sdk::worker::connection::*;
use spatialos_sdk::worker::commands::*;
use spatialos_sdk::worker::op::*;
use spatialos_sdk::worker::entity::Entity as WorkerEntity;
use spatialos_sdk::worker::RequestId;
use crate::EntityId;
use crate::component_registry::*;
use specs::shred::FetchMut;
use crate::*;
use crate::storage::*;
use std::collections::HashMap;
use crate::entities::*;

pub type SystemCommandSender<'a> = Write<'a, SystemCommandSenderImpl>;

type IntermediateCallback<O> = Box<FnOnce(&Resources, O) + Send + Sync>;

pub struct SystemCommandSenderImpl {
	create_entity_callbacks: HashMap<RequestId<CreateEntityRequest>, IntermediateCallback<CreateEntityResponseOp>>,
	buffered_create_entity_requests: Vec<(WorkerEntity, IntermediateCallback<CreateEntityResponseOp>)>
}

// TODO expose parameters like timeout
impl SystemCommandSenderImpl {

	pub fn create_entity<F>(
		&mut self, 
		entity: WorkerEntity,
		callback: F) 
	where 
		F: 'static + FnOnce(&Resources, Result<EntityId, StatusCode<EntityId>>) + Send + Sync
	{
		self.buffered_create_entity_requests.push((entity, Box::new(|res, response_op| {
			callback(res, SystemCommandSenderImpl::status_code_to_result(response_op.status_code));
		})));
	}

	pub(crate) fn got_create_entity_response(&mut self, res: &Resources, response_op: CreateEntityResponseOp) {
		match self.create_entity_callbacks.remove(&response_op.request_id) {
			Some(callback) => callback(res, response_op),
			None => println!("Unknown request ID: {:?}", response_op.request_id)
		}
	}

	pub(crate) fn flush_requests(&mut self, connection: &mut WorkerConnection) {
		for (entity, callback) in self.buffered_create_entity_requests.drain(..) {
			let request_id = connection.send_create_entity_request(entity, None, Default::default());
			self.create_entity_callbacks.insert(request_id, callback);
		}
	}

	fn status_code_to_result<T>(status_code: StatusCode<T>) -> Result<T, StatusCode<T>> {
		match status_code {
			StatusCode::Success(response) => {
				Ok(response)
			},
			other => {
				Err(other)
			}
		}
	}
}

impl Default for SystemCommandSenderImpl {
	fn default() -> Self {
		SystemCommandSenderImpl {
			create_entity_callbacks: HashMap::new(),
			buffered_create_entity_requests: Vec::new()
		}
	}
}
