use crate::component_registry::ComponentRegistry;
use crate::entities::SpatialEntity;
use crate::SpatialComponent;
use hibitset::{BitSet, BitSetAnd, BitSetLike};
use spatialos_sdk::worker::component::Component as WorkerComponent;
use spatialos_sdk::worker::Authority;
use specs::join::BitAnd;
use specs::prelude::{Component, Join, ReadStorage, Resources, SystemData, WriteStorage};
use specs::shred::{Fetch, ResourceId};
use specs::storage::{DistinctStorage, UnprotectedStorage};
use specs::world::Index;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

pub struct SpatialUnprotectedStorage<T, U>(U, PhantomData<T>)
where
    T: 'static + WorkerComponent,
    U: UnprotectedStorage<SpatialComponent<T>> + Default;

impl<T, U> UnprotectedStorage<SpatialComponent<T>> for SpatialUnprotectedStorage<T, U>
where
    T: 'static + WorkerComponent,
    U: UnprotectedStorage<SpatialComponent<T>> + Default,
{
    unsafe fn clean<B>(&mut self, has: B)
    where
        B: BitSetLike,
    {
        self.0.clean(has);
    }

    unsafe fn get(&self, id: Index) -> &SpatialComponent<T> {
        self.0.get(id)
    }

    unsafe fn get_mut(&mut self, id: Index) -> &mut SpatialComponent<T> {
        self.0.get_mut(id)
    }

    unsafe fn insert(&mut self, id: Index, v: SpatialComponent<T>) {
        self.0.insert(id, v);
    }

    unsafe fn remove(&mut self, id: Index) -> SpatialComponent<T> {
        self.0.remove(id)
    }
}

unsafe impl<T, U> DistinctStorage for SpatialUnprotectedStorage<T, U>
where
    T: 'static + WorkerComponent,
    U: UnprotectedStorage<SpatialComponent<T>> + Default,
{
}

impl<T, U> Default for SpatialUnprotectedStorage<T, U>
where
    T: 'static + WorkerComponent,
    U: UnprotectedStorage<SpatialComponent<T>> + Default,
{
    fn default() -> Self {
        ComponentRegistry::register_component::<T>();
        SpatialUnprotectedStorage(Default::default(), PhantomData)
    }
}

pub type SpatialReadStorage<'a, T> = ReadStorage<'a, SpatialComponent<T>>;

pub struct SpatialWriteStorage<'a, T: 'static + WorkerComponent> {
    data: WriteStorage<'a, SpatialComponent<T>>,
    authority: Fetch<'a, AuthorityBitSet<T>>,
}

impl<'a, T: 'static + WorkerComponent> Deref for SpatialWriteStorage<'a, T> {
    type Target = WriteStorage<'a, SpatialComponent<T>>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<'a, T: 'static + WorkerComponent> DerefMut for SpatialWriteStorage<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<'a, T: 'static + WorkerComponent> SystemData<'a> for SpatialWriteStorage<'a, T>
where
    T: 'static + WorkerComponent,
{
    fn setup(res: &mut Resources) {
        WriteStorage::<SpatialComponent<T>>::setup(res);
    }

    fn fetch(res: &'a Resources) -> Self {
        SpatialWriteStorage {
            data: WriteStorage::<SpatialComponent<T>>::fetch(res),
            authority: res.fetch(),
        }
    }

    fn reads() -> Vec<ResourceId> {
        vec![]
    }

    fn writes() -> Vec<ResourceId> {
        let mut writes = WriteStorage::<SpatialComponent<T>>::writes();
        writes.push(ResourceId::new::<AuthorityBitSet<T>>());
        writes
    }
}

impl<'a, 'e, T> Join for &'a mut SpatialWriteStorage<'e, T>
where
    T: 'static + WorkerComponent,
{
    type Mask = BitSetAnd<&'a BitSet, &'a BitSet>;
    type Type = &'a mut SpatialComponent<T>;
    type Value = &'a mut <SpatialComponent<T> as Component>::Storage;

    unsafe fn open(self) -> (Self::Mask, Self::Value) {
        let storage = &mut self.data;
        let (mask, value) = storage.open();
        ((&self.authority.mask, mask).and(), value)
    }

    unsafe fn get(v: &mut Self::Value, i: Index) -> &'a mut SpatialComponent<T> {
        <&'a mut WriteStorage<'a, SpatialComponent<T>> as Join>::get(v, i)
    }
}

pub struct AuthorityBitSet<T: WorkerComponent> {
    mask: BitSet,
    _phantom: PhantomData<T>,
}

impl<T: WorkerComponent> AuthorityBitSet<T> {
    pub(crate) fn new() -> Self {
        AuthorityBitSet {
            mask: BitSet::new(),
            _phantom: PhantomData,
        }
    }

    pub(crate) fn set_authority(&mut self, e: SpatialEntity, authority: Authority) {
        if authority == Authority::NotAuthoritative {
            self.mask.remove(e.specs_entity().id());
        } else {
            self.mask.add(e.specs_entity().id());
        }
    }
}
