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
use spatialos_sdk::worker::RequestId;
use crate::EntityId;
use crate::component_registry::*;
use specs::shred::FetchMut;
use crate::*;
use crate::storage::*;
use std::collections::HashMap;

pub type CommandRequests<'a, T> = WriteStorage<'a, CommandResponder<T>>;

pub struct CommandResponder<T: SpatialComponent> {
	requests: Vec<(RequestId<IncomingCommandRequest>, T::CommandRequest)>,
	responses: Vec<(RequestId<IncomingCommandRequest>, T::CommandResponse)>
}

impl<T: 'static + SpatialComponent> Component for CommandResponder<T> {
    type Storage = HashMapStorage<Self>;
}

impl<T: SpatialComponent> CommandResponder<T> {
	pub(crate) fn on_request(&mut self, request_id: RequestId<IncomingCommandRequest>, request: T::CommandRequest) {
		self.requests.push((request_id, request));
	}

	pub fn respond<F>(&mut self, mut responder: F) where F: FnMut(&T::CommandRequest) -> Option<T::CommandResponse> {
		let mut requests_left = Vec::new();
		for (request_id, request) in self.requests.drain(..) {
			match responder(&request) {
				Some(response) => self.responses.push((request_id, response)),
				None => requests_left.push((request_id, request))
			}
		}

		self.requests = requests_left;
	}

	pub(crate) fn flush_responses(&mut self, connection: &mut WorkerConnection) {
		for (request_id, response) in self.responses.drain(..) {
			connection.send_command_response::<T>(request_id, response);
		}
	}
}

pub type CommandSender<'a, T> = WriteAndRegisterComponent<'a, CommandSenderImpl<T>, T>;

pub struct CommandSenderImpl<T: SpatialComponent> {
	callbacks: HashMap<RequestId<OutgoingCommandRequest>, Box<FnOnce(&Resources, CommandResponseOp) + Send + Sync>>,
	buffered_requests: Vec<(EntityId, T::CommandRequest, Box<FnOnce(&Resources, CommandResponseOp) + Send + Sync>)>
}

impl<T: 'static + SpatialComponent> CommandSenderImpl<T> {
	pub fn send_command<F, G>(&mut self, 
			entity_id: EntityId,
			request: T::CommandRequest,
			success: F,
			failure: G) 
	where 
		F: 'static + FnOnce(&Resources, &T::CommandResponse) + Send + Sync,
		G: 'static + FnOnce(StatusCode<CommandResponse>) + Send + Sync
	{
		self.buffered_requests.push((entity_id, request, Box::new(|res: &Resources, response_op| {
			match response_op.response {
				StatusCode::Success(response) => {
					let response = response.get::<T>().unwrap();
					success(res, response);
				},
				other => {
					failure(other);
				}
			}
		})));
	}

	pub(crate) fn got_command_response(&mut self, res: &Resources, response_op: CommandResponseOp) {
		match self.callbacks.remove(&response_op.request_id) {
			Some(callback) => callback(res, response_op),
			None => println!("Unknown request ID: {:?}", response_op.request_id)
		}
	}

	pub(crate) fn flush_requests(&mut self, connection: &mut WorkerConnection) {
		for (entity_id, request, callback) in self.buffered_requests.drain(..) {
			// TODO: Default command params like timeout
			let request_id = connection.send_command_request::<T>(entity_id.0, request, None, Default::default());
			self.callbacks.insert(request_id, callback);
		}
	}
}

impl<T: SpatialComponent> Default for CommandSenderImpl<T> {
	fn default() -> Self {
		CommandSenderImpl {
			callbacks: HashMap::new(),
			buffered_requests: Vec::new()
		}
	}
}
