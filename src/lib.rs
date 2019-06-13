pub mod commands;
mod component_registry;
pub mod entities;
mod spatial_reader;
mod spatial_writer;
mod storage;
pub mod system_commands;

pub use commands::{CommandRequests, CommandSender};
pub use entities::{SpatialEntities, SpatialEntity};
pub use spatial_reader::SpatialReaderSystem;
pub use spatial_writer::SpatialWriterSystem;
pub use storage::{SpatialReadStorage, SpatialWriteStorage};
pub use system_commands::SystemCommandSender;

use spatialos_sdk::worker::component::Component as WorkerComponent;
use spatialos_sdk::worker::component::{ComponentUpdate, TypeConversion, UpdateParameters};
use spatialos_sdk::worker::connection::{Connection, WorkerConnection};
use spatialos_sdk::worker::internal::schema::SchemaComponentUpdate;
use spatialos_sdk::worker::EntityId;
use specs::prelude::{Component, VecStorage};
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub struct SpatialComponent<T: WorkerComponent + Debug> {
    value: T,
    value_is_dirty: bool,
    current_update: Option<T::Update>,
}

impl<T: WorkerComponent + TypeConversion + Debug> SpatialComponent<T> {
    pub(crate) fn new(value: T) -> SpatialComponent<T> {
        SpatialComponent {
            value,
            value_is_dirty: false,
            current_update: None,
        }
    }

    pub(crate) fn replicate(&mut self, connection: &mut WorkerConnection, entity_id: EntityId) {
        let update = {
            if self.value_is_dirty {
                self.value_is_dirty = false;
                Some(self.to_update())
            } else {
                self.current_update.take()
            }
        };

        if let Some(update) = update {
            connection.send_component_update::<T>(entity_id, update, UpdateParameters::default());
        }
    }

    // TODO - this is really bad as it seriliases then deserialises.
    fn to_update(&self) -> T::Update {
        let schema_update = SchemaComponentUpdate::new(T::ID);
        let mut fields = schema_update.fields();
        T::to_type(&self.value, &mut fields).unwrap();

        T::Update::from_type(&fields).unwrap()
    }

    pub(crate) fn apply_update_to_value(&mut self, update: T::Update) {
        self.value.merge(update);
    }

    pub fn send_update(&mut self, update: T::Update) {
        if self.value_is_dirty {
            panic!("Attempt to send update to component which has already been mutably dereferenced. Id {}", T::ID);
        }

        self.apply_update_to_value(update.clone());

        match &mut self.current_update {
            Some(current_update) => current_update.merge(update),
            None => self.current_update = Some(update),
        }
    }
}

impl<T: WorkerComponent + Debug> Deref for SpatialComponent<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T: WorkerComponent + Debug> DerefMut for SpatialComponent<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        if self.current_update.is_some() {
            panic!("Attempt to mutably dereference a component which has already had an update applied to it. Id {}", T::ID);
        }

        self.value_is_dirty = true;
        &mut self.value
    }
}

impl<T: 'static + WorkerComponent> Component for SpatialComponent<T> {
    type Storage = VecStorage<Self>;
}
