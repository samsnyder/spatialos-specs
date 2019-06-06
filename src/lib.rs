use spatialos_sdk::worker::component::Component as SpatialComponent;
use spatialos_sdk::worker::component::TypeConversion;
use std::ops::{DerefMut, Deref};
use specs::prelude::*;
use spatialos_sdk::worker::op::*;
use specs::shred::{Fetch, ResourceId, SystemData, Resource};
use specs::storage::MaskedStorage;
use std::marker::PhantomData;
use std::fmt::Debug;
use spatialos_sdk::worker::EntityId;
use spatialos_sdk::worker::internal::schema::SchemaComponentUpdate;

pub mod world;
pub mod storage;

#[derive(Debug)]
pub struct SynchronisedComponent<T: SpatialComponent + Debug> {
    value: T,
    is_dirty: bool
}

impl<T: SpatialComponent + TypeConversion + Debug> SynchronisedComponent<T> {
	pub fn new(value: T) -> SynchronisedComponent<T> {
		SynchronisedComponent {
			value,
            is_dirty: false
		}
	}

    pub(crate) fn get_and_clear_dity_bit(&mut self) -> bool {
        let is_dirty = self.is_dirty;
        self.is_dirty = false;
        is_dirty
    }

    // TODO - this is really bad as it seriliases then deserialises.
    pub(crate) fn to_update(&self) -> T::Update {
        let schema_update = SchemaComponentUpdate::new(T::ID);
        let mut fields = schema_update.fields();
        T::to_type(&self.value, &mut fields);

        T::Update::from_type(&fields).unwrap()
    }
}

impl<T: SpatialComponent + Debug> Deref for SynchronisedComponent<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T: SpatialComponent + Debug> DerefMut for SynchronisedComponent<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.is_dirty = true;
        &mut self.value
    }
}

impl<T: 'static + Sync + Send + Debug> Component for SynchronisedComponent<T>
where
    T: SpatialComponent,
{
    type Storage = VecStorage<Self>;
}


pub(crate) struct WrappedEntityId(EntityId);

impl Component for WrappedEntityId {
    type Storage = VecStorage<Self>;
}

