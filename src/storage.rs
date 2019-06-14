use crate::component_registry::ComponentRegistry;
use crate::entities::SpatialEntity;
use crate::SpatialComponent;
use hibitset::{BitSet, BitSetAnd, BitSetLike};
use spatialos_sdk::worker::component::Component as WorkerComponent;
use spatialos_sdk::worker::Authority;
use specs::join::BitAnd;
use specs::prelude::{Component, Join, ReadStorage, Resources, Storage, SystemData, WriteStorage, Read};
use specs::shred::{Fetch, FetchMut, ResourceId};
use specs::storage::{
    AntiStorage, DistinctStorage, InsertResult, MaskedStorage, UnprotectedStorage,
};
use specs::world::{EntitiesRes, Index};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut, Not};

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

// // pub type SpatialReadStorage<'a, T> =
// //     SpatialStorage<'a, T, Fetch<'a, MaskedStorage<SpatialComponent<T>>>>;

// // impl<'a, T> SystemData<'a> for SpatialReadStorage<'a, T>
// // where
// //     T: 'static + WorkerComponent,
// // {
// //     fn setup(res: &mut Resources) {
// //         ComponentRegistry::register_component::<T>(res);
// //     }

// //     fn fetch(res: &'a Resources) -> Self {
// //         SpatialStorage::new(res.fetch(), res.fetch(), res.fetch())
// //     }

// //     fn reads() -> Vec<ResourceId> {
// //         ReadStorage::<'a, SpatialComponent<T>>::reads()
// //     }

// //     fn writes() -> Vec<ResourceId> {
// //         ReadStorage::<'a, SpatialComponent<T>>::writes()
// //     }
// // }
// pub type SpatialWriteStorage<'a, T> =
//     SpatialStorage<'a, T, FetchMut<'a, MaskedStorage<SpatialComponent<T>>>>;

// // pub type SpatialWriteStorage2<'a, T> = (WriteStorage<'a, SpatialComponent<T>>, Read<'a, AuthorityBitSet<T>>);

pub struct SpatialWriteStorage<'a, T: 'static + WorkerComponent>{
    data: WriteStorage<'a, SpatialComponent<T>>,
    authority: Fetch<'a, AuthorityBitSet<T>>
}

impl<'a, T: 'static + WorkerComponent> Deref for SpatialWriteStorage<'a, T> {
    type Target = 
}

impl<'a, T: 'static + WorkerComponent> SystemData<'a> for SpatialWriteStorage<'a, T>
where
    T: 'static + WorkerComponent,
{
    fn setup(res: &mut Resources) {
        WriteStorage::<SpatialComponent<T>>::setup(res);
    }

    fn fetch(res: &'a Resources) -> Self {
        SpatialWriteStorage2 {
            data: WriteStorage::<SpatialComponent<T>>::fetch(res),
            authority: res.fetch()
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

    // SAFETY: No unsafe code and no invariants to fulfill.
    unsafe fn open(self) -> (Self::Mask, Self::Value) {
        let storage = &mut self.data;
        let (mask, value) = storage.open();
        ((&self.authority.mask, mask).and(), value)
    }

    // TODO: audit unsafe
    unsafe fn get(v: &mut Self::Value, i: Index) -> &'a mut SpatialComponent<T> {
        <&'a mut WriteStorage<'a, SpatialComponent<T>> as Join>::get(v, i)
    }
}

// // SAFETY: This is safe because of the `DistinctStorage` guarantees.
// #[cfg(feature = "parallel")]
// unsafe impl<'a, 'e, T, D> ParJoin for &'a mut SpatialStorage<'e, T, D>
// where
//     T: 'static + WorkerComponent,
//     D: DerefMut<Target = MaskedStorage<SpatialComponent<T>>>,
//     T::Storage: Sync + DistinctStorage,
// {
// }



// impl<'a, T> SystemData<'a> for SpatialWriteStorage2<'a, T>
// where
//     T: 'static + WorkerComponent,
// {
//     fn setup(res: &mut Resources) {
//         WriteStorage::<T>::setup(res);
//     }

//     fn fetch(res: &'a Resources) -> Self {
//         (WriteStorage::<T>::fetch(res), res.fetch())
//     }

//     fn reads() -> Vec<ResourceId> {
//         WriteStorage::<T>::reads();
//     }

//     fn writes() -> Vec<ResourceId> {
//         WriteStorage::<T>::writes();
//     }
// }


// impl<'a, T> SystemData<'a> for SpatialWriteStorage<'a, T>
// where
//     T: 'static + WorkerComponent,
// {
//     fn setup(res: &mut Resources) {
//         // ComponentRegistry::register_component::<T>(res);
//     }

//     fn fetch(res: &'a Resources) -> Self {
//         SpatialStorage::new(res.fetch(), res.fetch_mut(), res.fetch())
//     }

//     fn reads() -> Vec<ResourceId> {
//         WriteStorage::<'a, SpatialComponent<T>>::reads()
//     }

//     fn writes() -> Vec<ResourceId> {
//         WriteStorage::<'a, SpatialComponent<T>>::writes()
//     }
// }

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

// impl<'a, T: WorkerComponent> Join for &'a SpatialWriteStorage2<'a, T> {
//     type Type = <&'a WriteStorage<'a, SpatialComponent<T>> as Join>::Type;
//     type Value = <&'a WriteStorage<'a, SpatialComponent<T>> as Join>::Value;
//     type Mask = <&'a WriteStorage<'a, SpatialComponent<T>> as Join>::Mask;

//     unsafe fn open(self) -> (Self::Mask, Self::Value) {
//         // self.entity_id_storage.open()
//     }

//     unsafe fn get(v: &mut Self::Value, idx: Index) -> &'a SpatialEntity {
//         // <&'a ReadStorage<'a, SpatialEntity> as Join>::get(v, idx)
//     }
// }

// #[cfg(feature = "parallel")]
// unsafe impl<'a> ParJoin for SpatialEntitiesSystemData<'a> {}



// /// A wrapper around the masked SpatialStorage and the generations vector.
// /// Can be used for safe lookup of components, insertions and removes.
// /// This is what `World::read/write` fetches for the user.
// pub struct SpatialStorage<'e, T, D>
// where
//     T: 'static + WorkerComponent,
// {
//     storage: Storage<'e, SpatialComponent<T>, D>,
//     authority: Fetch<'e, AuthorityBitSet<T>>,
// }

// impl<'e, T, D> SpatialStorage<'e, T, D>
// where
//     T: 'static + WorkerComponent,
// {
//     /// Creates a new `SpatialStorage` from a fetched allocator and a immutable or
//     /// mutable `MaskedStorage`, named `data`.
//     pub(crate) fn new(
//         entities: Fetch<'e, EntitiesRes>,
//         data: D,
//         authority: Fetch<'e, AuthorityBitSet<T>>,
//     ) -> SpatialStorage<'e, T, D> {
//         SpatialStorage {
//             storage: Storage::new(entities, data),
//             authority,
//         }
//     }
// }

// impl<'e, T, D> SpatialStorage<'e, T, D>
// where
//     T: 'static + WorkerComponent,
//     D: Deref<Target = MaskedStorage<SpatialComponent<T>>>,
// {
//     /// Gets the wrapped SpatialStorage.
//     pub fn unprotected_storage(&self) -> &<SpatialComponent<T> as Component>::Storage {
//         self.storage.unprotected_storage()
//     }

//     /// Returns the `EntitiesRes` resource fetched by this SpatialStorage.
//     /// **This does not have anything to do with the components inside.**
//     /// You only want to use this when implementing additional methods
//     /// for `SpatialStorage` via an extension trait.
//     pub fn fetched_entities(&self) -> &EntitiesRes {
//         self.storage.fetched_entities()
//     }

//     /// Tries to read the data associated with an `Entity`.
//     pub fn get(&self, e: SpatialEntity) -> Option<&SpatialComponent<T>> {
//         self.storage.get(e.specs_entity())
//     }

//     /// Computes the number of elements this `SpatialStorage` contains by counting the
//     /// bits in the bit set. This operation will never be performed in
//     /// constant time.
//     pub fn count(&self) -> usize {
//         self.storage.count()
//     }

//     /// Checks whether this `SpatialStorage` is empty. This operation is very cheap.
//     pub fn is_empty(&self) -> bool {
//         self.storage.is_empty()
//     }

//     /// Returns true if the SpatialStorage has a component for this entity, and that
//     /// entity is alive.
//     pub fn contains(&self, e: SpatialEntity) -> bool {
//         self.storage.contains(e.specs_entity())
//     }

//     /// Returns a reference to the bitset of this SpatialStorage which allows filtering
//     /// by the component type without actually getting the component.
//     pub fn mask(&self) -> &BitSet {
//         self.storage.mask()
//     }
// }

// impl<'e, T, D> SpatialStorage<'e, T, D>
// where
//     T: 'static + WorkerComponent,
//     D: DerefMut<Target = MaskedStorage<SpatialComponent<T>>>,
// {
//     /// Gets mutable access to the wrapped SpatialStorage.
//     ///
//     /// # Safety
//     ///
//     /// This is unsafe because modifying the wrapped SpatialStorage without also
//     /// updating the mask bitset accordingly can result in illegal memory
//     /// access.
//     pub unsafe fn unprotected_storage_mut(
//         &mut self,
//     ) -> &mut <SpatialComponent<T> as Component>::Storage {
//         self.storage.unprotected_storage_mut()
//     }

//     /// Tries to mutate the data associated with an `Entity`.
//     pub fn get_mut(&mut self, e: SpatialEntity) -> Option<&mut SpatialComponent<T>> {
//         self.storage.get_mut(e.specs_entity())
//     }

//     /// Inserts new data for a given `Entity`.
//     /// Returns the result of the operation as a `InsertResult<T>`
//     ///
//     /// If a component already existed for the given `Entity`, then it will
//     /// be overwritten with the new component. If it did overwrite, then the
//     /// result will contain `Some(T)` where `T` is the previous component.
//     pub fn insert(
//         &mut self,
//         e: SpatialEntity,
//         v: SpatialComponent<T>,
//     ) -> InsertResult<SpatialComponent<T>> {
//         self.storage.insert(e.specs_entity(), v)
//     }

//     /// Removes the data associated with an `Entity`.
//     pub fn remove(&mut self, e: SpatialEntity) -> Option<SpatialComponent<T>> {
//         self.storage.remove(e.specs_entity())
//     }

//     /// Clears the contents of the SpatialStorage.
//     pub fn clear(&mut self) {
//         self.storage.clear()
//     }

//     // /// Creates a draining SpatialStorage wrapper which can be `.join`ed
//     // /// to get a draining iterator.
//     // pub fn drain(&mut self) -> Drain<T> {
//     //     unimplemented!()
//     // }
// }

// // SAFETY: This is safe, since `T::SpatialStorage` is `DistinctStorage` and `Join::get`
// // only accesses the SpatialStorage and nothing else.
// unsafe impl<'a, T: 'static + WorkerComponent, D> DistinctStorage for SpatialStorage<'a, T, D> where
//     <SpatialComponent<T> as Component>::Storage: DistinctStorage
// {
// }

// impl<'a, 'e, T, D: 'a> Join for &'a SpatialStorage<'e, T, D>
// where
//     T: 'static + WorkerComponent,
//     D: Deref<Target = MaskedStorage<SpatialComponent<T>>>,
// {
//     type Mask = &'a BitSet;
//     type Type = &'a SpatialComponent<T>;
//     type Value = &'a <SpatialComponent<T> as Component>::Storage;

//     // SAFETY: No unsafe code and no invariants.
//     unsafe fn open(self) -> (Self::Mask, Self::Value) {
//         self.storage.open()
//     }

//     // SAFETY: Since we require that the mask was checked, an element for `i` must
//     // have been inserted without being removed.
//     unsafe fn get(v: &mut Self::Value, i: Index) -> &'a SpatialComponent<T> {
//         <&'a Storage<'a, SpatialComponent<T>, D> as Join>::get(v, i)
//     }
// }

// impl<'a, 'e, T, D> Not for &'a SpatialStorage<'e, T, D>
// where
//     T: 'static + WorkerComponent,
//     D: Deref<Target = MaskedStorage<SpatialComponent<T>>>,
// {
//     type Output = AntiStorage<'a>;

//     fn not(self) -> Self::Output {
//         self.storage.not()
//     }
// }

// // SAFETY: This is always safe because immutable access can in no case cause
// // memory issues, even if access to common memory occurs.
// #[cfg(feature = "parallel")]
// unsafe impl<'a, 'e, T, D> ParJoin for &'a SpatialStorage<'e, T, D>
// where
//     T: 'static + WorkerComponent,
//     D: Deref<Target = MaskedStorage<SpatialComponent<T>>>,
//     T::Storage: Sync,
// {
// }

// impl<'a, 'e, T, D: 'a> Join for &'a mut SpatialStorage<'e, T, D>
// where
//     T: 'static + WorkerComponent,
//     D: DerefMut<Target = MaskedStorage<SpatialComponent<T>>>,
// {
//     type Mask = BitSetAnd<&'a BitSet, &'a BitSet>;
//     // type Mask = &'a BitSet;
//     type Type = &'a mut SpatialComponent<T>;
//     type Value = &'a mut <SpatialComponent<T> as Component>::Storage;

//     // SAFETY: No unsafe code and no invariants to fulfill.
//     unsafe fn open(self) -> (Self::Mask, Self::Value) {
//         let storage = &mut self.storage;
//         let (mask, value) = storage.open();
//         ((&self.authority.mask, mask).and(), value)
//     }

//     // TODO: audit unsafe
//     unsafe fn get(v: &mut Self::Value, i: Index) -> &'a mut SpatialComponent<T> {
//         <&'a mut Storage<'a, SpatialComponent<T>, D> as Join>::get(v, i)
//     }
// }

// // SAFETY: This is safe because of the `DistinctStorage` guarantees.
// #[cfg(feature = "parallel")]
// unsafe impl<'a, 'e, T, D> ParJoin for &'a mut SpatialStorage<'e, T, D>
// where
//     T: 'static + WorkerComponent,
//     D: DerefMut<Target = MaskedStorage<SpatialComponent<T>>>,
//     T::Storage: Sync + DistinctStorage,
// {
// }
