pub mod commands;
mod component_registry;
pub mod entities;
pub mod spatial_reader;
pub mod spatial_writer;
pub mod storage;
pub mod system_commands;

use crate::component_registry::ComponentRegistry;
use spatialos_sdk::worker::component::Component as SpatialComponent;
use spatialos_sdk::worker::component::TypeConversion;
use spatialos_sdk::worker::internal::schema::SchemaComponentUpdate;
use specs::prelude::{Component, Resources, SystemData, VecStorage, Write};
use specs::shred::{Resource, ResourceId};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

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
        T::to_type(&self.value, &mut fields).unwrap();

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

pub struct WriteAndRegisterComponent<'a, T: 'a + Resource, C: SpatialComponent> {
    resource: Write<'a, T>,
    phantom: PhantomData<C>,
}

impl<'a, T, C: SpatialComponent> Deref for WriteAndRegisterComponent<'a, T, C>
where
    T: Resource,
{
    type Target = T;

    fn deref(&self) -> &T {
        self.resource.deref()
    }
}

impl<'a, T, C: SpatialComponent> DerefMut for WriteAndRegisterComponent<'a, T, C>
where
    T: Resource,
{
    fn deref_mut(&mut self) -> &mut T {
        self.resource.deref_mut()
    }
}

impl<'a, T, C: SpatialComponent> SystemData<'a> for WriteAndRegisterComponent<'a, T, C>
where
    C: 'static + SpatialComponent,
    T: Resource + Default,
{
    fn setup(res: &mut Resources) {
        ComponentRegistry::register_component::<C>(res);
        Write::<T>::setup(res);
    }

    fn fetch(res: &'a Resources) -> Self {
        WriteAndRegisterComponent {
            resource: Write::fetch(res),
            phantom: PhantomData,
        }
    }

    fn reads() -> Vec<ResourceId> {
        Write::<T>::reads()
    }

    fn writes() -> Vec<ResourceId> {
        Write::<T>::writes()
    }
}
