use specs::prelude::*;
use specs::world::*;
use specs::storage::*;
use spatialos_sdk::worker::component::Component as SpatialComponent;

use std::{
    self,
    marker::PhantomData,
    ops::{Deref, DerefMut, Not},
};

use hibitset::{BitSet, BitSetLike, BitSetNot};
use specs::shred::{CastFrom, Fetch, FetchMut, ResourceId};

use crate::*;





pub type SpatialReadStorage<'a, T> = SpatialStorage<'a, SynchronisedComponent<T>, Fetch<'a, MaskedStorage<SynchronisedComponent<T>>>>;

impl<'a, T> SystemData<'a> for SpatialReadStorage<'a, T>
where
    T: 'static + SpatialComponent + Send + Sync + Debug,
{
    fn setup(res: &mut Resources) {
        ReadStorage::<'a, SynchronisedComponent<T>>::setup(res);
    }

    fn fetch(res: &'a Resources) -> Self {
    	SpatialStorage::new(res.fetch(), res.fetch())
    }

    fn reads() -> Vec<ResourceId> {
        ReadStorage::<'a, SynchronisedComponent<T>>::reads()
    }

    fn writes() -> Vec<ResourceId> {
        ReadStorage::<'a, SynchronisedComponent<T>>::writes()
    }
}


pub type SpatialWriteStorage<'a, T> = SpatialStorage<'a, SynchronisedComponent<T>, FetchMut<'a, MaskedStorage<SynchronisedComponent<T>>>>;

impl<'a, T> SystemData<'a> for SpatialWriteStorage<'a, T>
where
    T: 'static + SpatialComponent + Send + Sync + Debug,
{
    fn setup(res: &mut Resources) {
        WriteStorage::<'a, SynchronisedComponent<T>>::setup(res);
    }

    fn fetch(res: &'a Resources) -> Self {
    	SpatialStorage::new(res.fetch(), res.fetch_mut())
    }

    fn reads() -> Vec<ResourceId> {
        WriteStorage::<'a, SynchronisedComponent<T>>::reads()
    }

    fn writes() -> Vec<ResourceId> {
        WriteStorage::<'a, SynchronisedComponent<T>>::writes()
    }
}







/// A wrapper around the masked SpatialStorage and the generations vector.
/// Can be used for safe lookup of components, insertions and removes.
/// This is what `World::read/write` fetches for the user.
pub struct SpatialStorage<'e, T, D> {
    storage: Storage<'e, T, D>
}

impl<'e, T, D> SpatialStorage<'e, T, D> {
    /// Creates a new `SpatialStorage` from a fetched allocator and a immutable or
    /// mutable `MaskedStorage`, named `data`.
    pub fn new(entities: Fetch<'e, EntitiesRes>, data: D) -> SpatialStorage<'e, T, D> {
        SpatialStorage {
            storage: Storage::new(entities, data)
        }
    }
}

impl<'e, T, D> SpatialStorage<'e, T, D>
where
    T: Component,
    D: Deref<Target = MaskedStorage<T>>,
{
    /// Gets the wrapped SpatialStorage.
    pub fn unprotected_storage(&self) -> &T::Storage {
        self.storage.unprotected_storage()
    }

    /// Returns the `EntitiesRes` resource fetched by this SpatialStorage.
    /// **This does not have anything to do with the components inside.**
    /// You only want to use this when implementing additional methods
    /// for `SpatialStorage` via an extension trait.
    pub fn fetched_entities(&self) -> &EntitiesRes {
        self.storage.fetched_entities()
    }

    /// Tries to read the data associated with an `Entity`.
    pub fn get(&self, e: Entity) -> Option<&T> {
        self.storage.get(e)
    }

    /// Computes the number of elements this `SpatialStorage` contains by counting the
    /// bits in the bit set. This operation will never be performed in
    /// constant time.
    pub fn count(&self) -> usize {
        self.storage.count()
    }

    /// Checks whether this `SpatialStorage` is empty. This operation is very cheap.
    pub fn is_empty(&self) -> bool {
        self.storage.is_empty()
    }

    /// Returns true if the SpatialStorage has a component for this entity, and that
    /// entity is alive.
    pub fn contains(&self, e: Entity) -> bool {
        self.storage.contains(e)
    }

    /// Returns a reference to the bitset of this SpatialStorage which allows filtering
    /// by the component type without actually getting the component.
    pub fn mask(&self) -> &BitSet {
        self.storage.mask()
    }
}

impl<'e, T, D> SpatialStorage<'e, T, D>
where
    T: Component,
    D: DerefMut<Target = MaskedStorage<T>>,
{
    /// Gets mutable access to the wrapped SpatialStorage.
    ///
    /// # Safety
    ///
    /// This is unsafe because modifying the wrapped SpatialStorage without also
    /// updating the mask bitset accordingly can result in illegal memory
    /// access.
    pub unsafe fn unprotected_storage_mut(&mut self) -> &mut T::Storage {
        self.storage.unprotected_storage_mut()
    }

    /// Tries to mutate the data associated with an `Entity`.
    pub fn get_mut(&mut self, e: Entity) -> Option<&mut T> {
        self.storage.get_mut(e)
    }

    /// Inserts new data for a given `Entity`.
    /// Returns the result of the operation as a `InsertResult<T>`
    ///
    /// If a component already existed for the given `Entity`, then it will
    /// be overwritten with the new component. If it did overwrite, then the
    /// result will contain `Some(T)` where `T` is the previous component.
    pub fn insert(&mut self, e: Entity, mut v: T) -> InsertResult<T> {
        self.storage.insert(e, v)
    }

    /// Removes the data associated with an `Entity`.
    pub fn remove(&mut self, e: Entity) -> Option<T> {
        self.storage.remove(e)
    }

    /// Clears the contents of the SpatialStorage.
    pub fn clear(&mut self) {
        self.storage.clear()
    }

    // /// Creates a draining SpatialStorage wrapper which can be `.join`ed
    // /// to get a draining iterator.
    // pub fn drain(&mut self) -> Drain<T> {
    //     unimplemented!()
    // }
}

// SAFETY: This is safe, since `T::SpatialStorage` is `DistinctStorage` and `Join::get`
// only accesses the SpatialStorage and nothing else.
unsafe impl<'a, T: Component, D> DistinctStorage for SpatialStorage<'a, T, D> where
    T::Storage: DistinctStorage
{
}

impl<'a, 'e, T, D: 'a> Join for &'a SpatialStorage<'e, T, D>
where
    T: Component,
    D: Deref<Target = MaskedStorage<T>>,
{
    type Mask = &'a BitSet;
    type Type = &'a T;
    type Value = &'a T::Storage;

    // SAFETY: No unsafe code and no invariants.
    unsafe fn open(self) -> (Self::Mask, Self::Value) {
        self.storage.open()
    }

    // SAFETY: Since we require that the mask was checked, an element for `i` must
    // have been inserted without being removed.
    unsafe fn get(v: &mut Self::Value, i: Index) -> &'a T {
        <&'a Storage<'a, T, D> as Join>::get(v, i)
    }
}

impl<'a, 'e, T, D> Not for &'a SpatialStorage<'e, T, D>
where
    T: Component,
    D: Deref<Target = MaskedStorage<T>>,
{
    type Output = AntiStorage<'a>;

    fn not(self) -> Self::Output {
        self.storage.not()
    }
}

// SAFETY: This is always safe because immutable access can in no case cause
// memory issues, even if access to common memory occurs.
#[cfg(feature = "parallel")]
unsafe impl<'a, 'e, T, D> ParJoin for &'a SpatialStorage<'e, T, D>
where
    T: Component,
    D: Deref<Target = MaskedStorage<T>>,
    T::Storage: Sync,
{
}

impl<'a, 'e, T, D: 'a> Join for &'a mut SpatialStorage<'e, T, D>
where
    T: Component,
    D: DerefMut<Target = MaskedStorage<T>>,
{
    type Mask = &'a BitSet;
    type Type = &'a mut T;
    type Value = &'a mut T::Storage;

    // SAFETY: No unsafe code and no invariants to fulfill.
    unsafe fn open(self) -> (Self::Mask, Self::Value) {
    	let storage = &mut self.storage;
        storage.open()
    }

    // TODO: audit unsafe
    unsafe fn get(v: &mut Self::Value, i: Index) -> &'a mut T {
        <&'a mut Storage<'a, T, D> as Join>::get(v, i)
    }
}

// SAFETY: This is safe because of the `DistinctStorage` guarantees.
#[cfg(feature = "parallel")]
unsafe impl<'a, 'e, T, D> ParJoin for &'a mut SpatialStorage<'e, T, D>
where
    T: Component,
    D: DerefMut<Target = MaskedStorage<T>>,
    T::Storage: Sync + DistinctStorage,
{
}