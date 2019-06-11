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
use hibitset::{BitSet, BitSetAnd, BitSetLike, BitSetNot};
use specs::world::Index;
use specs::world::EntitiesRes;

// This must stay immutable
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SpatialEntity {
    id: EntityId,
    specs_entity: Entity
}

impl SpatialEntity {
    pub(crate) fn new(entity_id: EntityId, specs_entity: Entity) -> SpatialEntity {
        SpatialEntity {
            id: entity_id,
            specs_entity
        }
    }

    pub(crate) fn entity_id(self) -> EntityId {
        self.id
    }

    pub(crate) fn specs_entity(self) -> Entity {
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


pub type SpatialEntities<'a> = SpatialEntitiesSystemData<Fetch<'a, SpatialEntitiesRes>, ReadStorage<'a, SpatialEntity>>;
pub(crate) type SpatialEntitiesWrite<'a> = SpatialEntitiesSystemData<FetchMut<'a, SpatialEntitiesRes>, WriteStorage<'a, SpatialEntity>>;

#[derive(Debug, Default)]
pub struct SpatialEntitiesRes {
	entities: HashMap<EntityId, SpatialEntity>
}

pub struct SpatialEntitiesSystemData<F, T> {
	spatial_entities_res: F,
	entity_id_storage: T
}

impl<'a> SpatialEntities<'a> {
	pub(crate) fn get_entity(&self, entity_id: EntityId) -> Option<SpatialEntity> {
		match self.spatial_entities_res.entities.get(&entity_id) {
			Some(entity) => Some(entity.clone()),
			None => None
		}
	}
}

impl<'a> SpatialEntitiesWrite<'a> {
	pub(crate) fn got_new_entity(&mut self, res: &Resources, entity_id: EntityId) {
		let specs_entity = Entities::fetch(res).create();
        let entity = SpatialEntity::new(entity_id, specs_entity);

		self.spatial_entities_res.entities.insert(entity_id, entity);
		self.entity_id_storage.insert(specs_entity, entity);
	}

	pub(crate) fn remove_entity(&mut self, entity_id: EntityId) {

	}

	pub(crate) fn get_entity(&self, entity_id: EntityId) -> Option<SpatialEntity> {
		match self.spatial_entities_res.entities.get(&entity_id) {
			Some(entity) => Some(entity.clone()),
			None => None
		}
	}
}

impl<'a> SystemData<'a> for SpatialEntities<'a>
{
    fn setup(res: &mut Resources) {
    	Read::<SpatialEntitiesRes>::setup(res);
        ReadStorage::<SpatialEntity>::setup(res);
    }

    fn fetch(res: &'a Resources) -> Self {
        SpatialEntitiesSystemData {
        	spatial_entities_res: res.fetch(),
        	entity_id_storage: ReadStorage::<'a, SpatialEntity>::fetch(res)
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

impl<'a> SystemData<'a> for SpatialEntitiesWrite<'a>
{
    fn setup(res: &mut Resources) {
    	Write::<SpatialEntitiesRes>::setup(res);
        WriteStorage::<SpatialEntity>::setup(res);
    }

    fn fetch(res: &'a Resources) -> Self {
        SpatialEntitiesSystemData {
        	spatial_entities_res: res.fetch_mut(),
        	entity_id_storage: WriteStorage::<'a, SpatialEntity>::fetch(res)
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