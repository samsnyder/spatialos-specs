use spatialos_sdk::worker::component::Component as SpatialComponent;
use spatialos_sdk::worker::component::TypeConversion;
use spatialos_sdk::worker::internal::schema::SchemaComponentUpdate;
use spatialos_sdk::worker::op::*;
use spatialos_sdk::worker::EntityId;
use spatialos_sdk::worker::*;
use specs::prelude::*;
use specs::shred::{Fetch, Resource, ResourceId, SystemData};
use specs::storage::MaskedStorage;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

mod component_registry;
pub mod spatial_reader;
pub mod spatial_writer;
pub mod storage;

#[derive(Debug)]
pub struct SynchronisedComponent<T: SpatialComponent + Debug> {
    value: T,
    is_dirty: bool,
}

impl<T: SpatialComponent + TypeConversion + Debug> SynchronisedComponent<T> {
    pub fn new(value: T) -> SynchronisedComponent<T> {
        SynchronisedComponent {
            value,
            is_dirty: false,
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

    pub(crate) fn apply_update(&mut self, update: T::Update) {
        self.value.merge(update);
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

impl<T: 'static + SpatialComponent> Component for SynchronisedComponent<T> {
    type Storage = VecStorage<Self>;
}

pub(crate) struct WrappedEntityId(EntityId);

impl Component for WrappedEntityId {
    type Storage = VecStorage<Self>;
}

pub(crate) struct AuthorityBitSet<T: SpatialComponent> {
    mask: BitSet,
    _phantom: PhantomData<T>,
}

impl<T: SpatialComponent> AuthorityBitSet<T> {
    pub fn new() -> AuthorityBitSet<T> {
        AuthorityBitSet {
            mask: BitSet::new(),
            _phantom: PhantomData,
        }
    }

    pub fn set_authority(&mut self, e: Entity, authority: Authority) {
        if authority == Authority::NotAuthoritative {
            self.mask.remove(e.id());
        } else {
            self.mask.add(e.id());
        }
    }
}
