use spatialos_sdk::worker::EntityId as WorkerEntityId;
use specs::prelude::{
    Component, Entities, Entity, Join, Read, ReadStorage, Resources, SystemData, VecStorage,
    WriteStorage,
};
use specs::shred::{Fetch, ResourceId};
use specs::storage::MaskedStorage;
use specs::world::Index;
use std::collections::HashMap;
use std::ops::Deref;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EntityId(pub(crate) WorkerEntityId);

impl EntityId {
    pub fn id(self) -> WorkerEntityId {
        self.0
    }
}

impl Deref for EntityId {
    type Target = WorkerEntityId;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Component for EntityId {
    type Storage = VecStorage<Self>;
}

#[derive(Debug, Default)]
pub struct SpatialEntitiesRes {
    entities: HashMap<EntityId, Entity>,
}

impl SpatialEntitiesRes {
    pub(crate) fn got_new_entity(&mut self, res: &Resources, entity_id: EntityId) {
        let specs_entity = Entities::fetch(res).create();

        self.entities.insert(entity_id, specs_entity);
        WriteStorage::<EntityId>::fetch(res)
            .insert(specs_entity, entity_id)
            .expect("Error inserting new EntityId object.");
    }

    pub(crate) fn remove_entity(&mut self, res: &Resources, entity_id: EntityId) {
        let entity = self.entities.remove(&entity_id).unwrap();
        WriteStorage::<EntityId>::fetch(res).remove(entity);
        Entities::fetch(res)
            .delete(entity)
            .expect("Error deleting specs entity.");
    }

    pub fn get_entity(&self, entity_id: EntityId) -> Option<Entity> {
        match self.entities.get(&entity_id) {
            Some(entity) => Some(entity.clone()),
            None => None,
        }
    }
}

pub type EntityIds<'a> = EntityIdsSystemData<'a>;

#[doc(hidden)]
pub struct EntityIdsSystemData<'a> {
    spatial_entities_res: Fetch<'a, SpatialEntitiesRes>,
    entity_id_storage: ReadStorage<'a, EntityId>,
}

impl<'a> Deref for EntityIdsSystemData<'a> {
    type Target = Fetch<'a, SpatialEntitiesRes>;

    fn deref(&self) -> &Self::Target {
        &self.spatial_entities_res
    }
}

impl<'a> SystemData<'a> for EntityIdsSystemData<'a> {
    fn setup(res: &mut Resources) {
        Read::<SpatialEntitiesRes>::setup(res);
        ReadStorage::<EntityId>::setup(res);
    }

    fn fetch(res: &'a Resources) -> Self {
        EntityIdsSystemData {
            spatial_entities_res: res.fetch(),
            entity_id_storage: ReadStorage::<'a, EntityId>::fetch(res),
        }
    }

    fn reads() -> Vec<ResourceId> {
        vec![
            ResourceId::new::<SpatialEntitiesRes>(),
            ResourceId::new::<MaskedStorage<EntityId>>(),
        ]
    }

    fn writes() -> Vec<ResourceId> {
        vec![]
    }
}

impl<'a> Join for &'a EntityIdsSystemData<'a> {
    type Type = <&'a ReadStorage<'a, EntityId> as Join>::Type;
    type Value = <&'a ReadStorage<'a, EntityId> as Join>::Value;
    type Mask = <&'a ReadStorage<'a, EntityId> as Join>::Mask;

    unsafe fn open(self) -> (Self::Mask, Self::Value) {
        self.entity_id_storage.open()
    }

    unsafe fn get(v: &mut Self::Value, idx: Index) -> &'a EntityId {
        <&'a ReadStorage<'a, EntityId> as Join>::get(v, idx)
    }
}

#[cfg(feature = "parallel")]
unsafe impl<'a> ParJoin for EntityIdsSystemData<'a> {}

#[test]
fn entities_should_be_added_and_removed_successfully() {
    use specs::prelude::World;

    let mut world = World::new();

    type SystemData<'a> = (Entities<'a>, EntityIds<'a>);

    SystemData::setup(&mut world.res);

    {
        let (entities, entity_ids) = SystemData::fetch(&world.res);
        assert!((&entities, &entity_ids).join().next().is_none());
    }

    world
        .res
        .fetch_mut::<SpatialEntitiesRes>()
        .got_new_entity(&world.res, EntityId(WorkerEntityId::new(5)));

    {
        let (entities, entity_ids) = SystemData::fetch(&world.res);
        let (entity, entity_id) = (&entities, &entity_ids).join().next().unwrap();
        assert_eq!(5, entity_id.id().id);

        let fetched_entity = entity_ids
            .get_entity(EntityId(WorkerEntityId::new(5)))
            .unwrap();
        assert_eq!(entity, fetched_entity);
    }

    world
        .res
        .fetch_mut::<SpatialEntitiesRes>()
        .remove_entity(&world.res, EntityId(WorkerEntityId::new(5)));

    {
        let (entities, entity_ids) = SystemData::fetch(&world.res);
        assert!((&entities, &entity_ids).join().next().is_none());
        assert!(entity_ids
            .get_entity(EntityId(WorkerEntityId::new(5)))
            .is_none());
    }
}
