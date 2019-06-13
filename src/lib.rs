pub mod commands;
mod component_registry;
pub mod entities;
pub mod spatial_reader;
pub mod spatial_writer;
pub mod storage;
pub mod system_commands;

use crate::component_registry::ComponentRegistry;
use spatialos_sdk::worker::component::Component as SpatialComponent;
use spatialos_sdk::worker::component::{ComponentUpdate, TypeConversion, UpdateParameters};
use spatialos_sdk::worker::connection::{Connection, WorkerConnection};
use spatialos_sdk::worker::internal::schema::SchemaComponentUpdate;
use spatialos_sdk::worker::EntityId;
use specs::prelude::{Component, Resources, SystemData, VecStorage, Write};
use specs::shred::{Resource, ResourceId};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub struct SynchronisedComponent<T: SpatialComponent + Debug> {
    value: T,
    value_is_dirty: bool,
    current_update: Option<T::Update>,
}

impl<T: SpatialComponent + TypeConversion + Debug> SynchronisedComponent<T> {
    pub fn new(value: T) -> SynchronisedComponent<T> {
        SynchronisedComponent {
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

    pub fn apply_update(&mut self, update: T::Update) {
        if self.value_is_dirty {
            panic!("Attempt to apply update to component which has already been mutably dereferenced. Id {}", T::ID);
        }

        self.apply_update_to_value(update.clone());

        match &mut self.current_update {
            Some(current_update) => current_update.merge(update),
            None => self.current_update = Some(update),
        }
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
        if self.current_update.is_some() {
            panic!("Attempt to mutably dereference a component which has already had an update applied to it. Id {}", T::ID);
        }

        self.value_is_dirty = true;
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
