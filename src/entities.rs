use spatialos_sdk::worker::EntityId;
use specs::prelude::{
    Component, Entities, Entity, Join, Read, ReadStorage, Resources, SystemData, VecStorage, Write,
    WriteStorage,
};
use specs::shred::{Fetch, FetchMut, ResourceId};
use specs::storage::MaskedStorage;
use specs::world::Index;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

// This must stay immutable
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SpatialEntity {
    id: EntityId,
    specs_entity: Entity,
}

impl SpatialEntity {
    pub(crate) fn new(entity_id: EntityId, specs_entity: Entity) -> SpatialEntity {
        SpatialEntity {
            id: entity_id,
            specs_entity,
        }
    }

    pub fn entity_id(self) -> EntityId {
        self.id
    }

    pub fn specs_entity(self) -> Entity {
        self.specs_entity
    }
}

impl Component for SpatialEntity {
    type Storage = VecStorage<Self>;
}

impl From<SpatialEntity> for Entity {
    fn from(entity: SpatialEntity) -> Self {
        entity.specs_entity
    }
}

pub type SpatialEntities<'a> =
    SpatialEntitiesSystemData<Fetch<'a, SpatialEntitiesRes>, ReadStorage<'a, SpatialEntity>>;
pub(crate) type SpatialEntitiesWrite<'a> =
    SpatialEntitiesSystemData<FetchMut<'a, SpatialEntitiesRes>, WriteStorage<'a, SpatialEntity>>;

#[derive(Debug, Default)]
pub struct SpatialEntitiesRes {
    entities: HashMap<EntityId, SpatialEntity>,
}

impl SpatialEntitiesRes {
    pub(crate) fn got_new_entity(&mut self, res: &Resources, entity_id: EntityId) {
        let specs_entity = Entities::fetch(res).create();
        let entity = SpatialEntity::new(entity_id, specs_entity);

        self.entities.insert(entity_id, entity);
        WriteStorage::<SpatialEntity>::fetch(res)
            .insert(specs_entity, entity)
            .expect("Error inserting new SpatialEntity object.");
    }

    pub(crate) fn remove_entity(&mut self, res: &Resources, entity_id: EntityId) {
        let entity = self.entities.remove(&entity_id).unwrap();
        WriteStorage::<SpatialEntity>::fetch(res).remove(entity.specs_entity());
        Entities::fetch(res)
            .delete(entity.specs_entity())
            .expect("Error deleting specs entity.");
    }

    pub fn get_entity(&self, entity_id: EntityId) -> Option<SpatialEntity> {
        match self.entities.get(&entity_id) {
            Some(entity) => Some(entity.clone()),
            None => None,
        }
    }
}

#[doc(hidden)]
pub struct SpatialEntitiesSystemData<F, T> {
    spatial_entities_res: F,
    entity_id_storage: T,
}

impl<F, T> Deref for SpatialEntitiesSystemData<F, T> {
    type Target = F;

    fn deref(&self) -> &Self::Target {
        &self.spatial_entities_res
    }
}

impl<F, T> DerefMut for SpatialEntitiesSystemData<F, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.spatial_entities_res
    }
}

impl<'a> SystemData<'a> for SpatialEntities<'a> {
    fn setup(res: &mut Resources) {
        Read::<SpatialEntitiesRes>::setup(res);
        ReadStorage::<SpatialEntity>::setup(res);
    }

    fn fetch(res: &'a Resources) -> Self {
        SpatialEntitiesSystemData {
            spatial_entities_res: res.fetch(),
            entity_id_storage: ReadStorage::<'a, SpatialEntity>::fetch(res),
        }
    }

    fn reads() -> Vec<ResourceId> {
        vec![
            ResourceId::new::<SpatialEntitiesRes>(),
            ResourceId::new::<MaskedStorage<SpatialEntity>>(),
        ]
    }

    fn writes() -> Vec<ResourceId> {
        vec![]
    }
}

impl<'a> SystemData<'a> for SpatialEntitiesWrite<'a> {
    fn setup(res: &mut Resources) {
        Write::<SpatialEntitiesRes>::setup(res);
        WriteStorage::<SpatialEntity>::setup(res);
    }

    fn fetch(res: &'a Resources) -> Self {
        SpatialEntitiesSystemData {
            spatial_entities_res: res.fetch_mut(),
            entity_id_storage: WriteStorage::<'a, SpatialEntity>::fetch(res),
        }
    }

    fn reads() -> Vec<ResourceId> {
        vec![]
    }

    fn writes() -> Vec<ResourceId> {
        vec![
            ResourceId::new::<SpatialEntitiesRes>(),
            ResourceId::new::<MaskedStorage<SpatialEntity>>(),
        ]
    }
}

impl<'a> Join for &'a SpatialEntities<'a> {
    type Type = <&'a ReadStorage<'a, SpatialEntity> as Join>::Type;
    type Value = <&'a ReadStorage<'a, SpatialEntity> as Join>::Value;
    type Mask = <&'a ReadStorage<'a, SpatialEntity> as Join>::Mask;

    unsafe fn open(self) -> (Self::Mask, Self::Value) {
        self.entity_id_storage.open()
    }

    unsafe fn get(v: &mut Self::Value, idx: Index) -> &'a SpatialEntity {
        <&'a ReadStorage<'a, SpatialEntity> as Join>::get(v, idx)
    }
}

#[cfg(feature = "parallel")]
unsafe impl<'a> ParJoin for SpatialEntitiesSystemData<'a> {}
