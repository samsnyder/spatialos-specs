use crate::storage::*;
use crate::*;
use spatialos_sdk::worker::component::Component as SpatialComponent;
use spatialos_sdk::worker::component::VTable;
use spatialos_sdk::worker::component::{ComponentId, UpdateParameters};
use spatialos_sdk::worker::connection::*;
use spatialos_sdk::worker::op::*;
use spatialos_sdk::worker::*;
use spatialos_sdk::worker::*;
use specs::prelude::*;
use specs::world::*;
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;

pub(crate) struct ComponentRegistry {
    interfaces: HashMap<ComponentId, Box<ComponentDispatcherInterface + Send + Sync>>,
}

impl ComponentRegistry {
    fn new() -> ComponentRegistry {
        ComponentRegistry {
            interfaces: HashMap::new(),
        }
    }

    pub(crate) fn register_component<C: 'static + SpatialComponent>(res: &mut Resources) {
        res.entry::<ComponentRegistry>()
            .or_insert_with(|| ComponentRegistry::new())
            .register_component_on_self::<C>();
        res.insert(AuthorityBitSet::<C>::new());
    }

    fn register_component_on_self<C: 'static + SpatialComponent>(&mut self) {
        let interface = ComponentDispatcher::<C> {
            _phantom: PhantomData,
        };
        self.interfaces.insert(C::ID, Box::new(interface));
    }

    pub(crate) fn get_interface(
        &self,
        component_id: ComponentId,
    ) -> Option<&Box<ComponentDispatcherInterface + Send + Sync>> {
        self.interfaces.get(&component_id)
    }

    pub(crate) fn interfaces_iter(
        &self,
    ) -> impl Iterator<Item = &Box<ComponentDispatcherInterface + Send + Sync>> {
        self.interfaces.values()
    }
}

struct ComponentDispatcher<C: 'static + SpatialComponent + Sync + Send + Clone + Debug> {
    _phantom: PhantomData<C>,
}

pub(crate) trait ComponentDispatcherInterface {
    fn add_component<'b>(
        &self,
        res: &Resources,
        entity: Entity,
        add_component: AddComponentOp,
    );
    fn remove_component<'b>(
        &self,
        res: &Resources,
        entity: Entity
    );
    fn apply_component_update<'b>(
        &self,
        res: &Resources,
        entity: Entity,
        component_update: ComponentUpdateOp,
    );
    fn apply_authority_change<'b>(
        &self,
        res: &Resources,
        entity: Entity,
        authority_change: AuthorityChangeOp,
    );
    fn replicate(&self, res: &Resources, connection: &mut WorkerConnection);
}

impl<T: 'static + SpatialComponent + Sync + Send + Clone + Debug> ComponentDispatcherInterface
    for ComponentDispatcher<T>
{
    fn add_component<'b>(
        &self,
        res: &Resources,
        entity: Entity,
        add_component: AddComponentOp,
    ) {
        let mut storage: SpatialWriteStorage<T> = SpatialStorage::fetch(res);
        let data = add_component.get::<T>().unwrap().clone();

        storage.insert(entity, SynchronisedComponent::new(data)).unwrap();
    }

    fn remove_component<'b>(
        &self,
        res: &Resources,
        entity: Entity
    ) {
        let mut storage: SpatialWriteStorage<T> = SpatialStorage::fetch(res);
        storage.remove(entity);
    }

    fn apply_component_update<'b>(
        &self,
        res: &Resources,
        entity: Entity,
        component_update: ComponentUpdateOp,
    ) {
        let mut storage: SpatialWriteStorage<T> = SpatialStorage::fetch(res);
        let update = component_update.get::<T>().unwrap().clone();

        storage.get_mut(entity).unwrap().apply_update(update);
    }

    fn apply_authority_change<'b>(
        &self,
        res: &Resources,
        entity: Entity,
        authority_change: AuthorityChangeOp,
    ) {
        res.fetch_mut::<AuthorityBitSet<T>>()
            .set_authority(entity, authority_change.authority);
    }

    fn replicate(&self, res: &Resources, connection: &mut WorkerConnection) {
        let entity_ids: ReadStorage<WrappedEntityId> = Storage::fetch(res);
        let mut storage: SpatialWriteStorage<T> = SpatialStorage::fetch(res);

        for (entity_id, component) in (&entity_ids, &mut storage).join() {
            if component.get_and_clear_dity_bit() {
                connection.send_component_update::<T>(
                    entity_id.0,
                    component.to_update(),
                    UpdateParameters::default(),
                );
            }
        }
    }
}
