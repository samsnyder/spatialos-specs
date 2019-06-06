use spatialos_sdk::worker::component::Component as SpatialComponent;
use std::ops::{DerefMut, Deref};
use specs::prelude::*;
use spatialos_sdk::worker::op::*;
use specs::shred::{Fetch, ResourceId, SystemData, Resource};
use specs::storage::MaskedStorage;
use std::marker::PhantomData;
use std::fmt::Debug;

pub mod world;
pub mod storage;

#[derive(Debug)]
pub struct SynchronisedComponent<T: SpatialComponent + Debug> {
    value: T,
    is_dirty: bool
}

impl<T: SpatialComponent + Debug> SynchronisedComponent<T> {
	pub fn new(value: T) -> SynchronisedComponent<T> {
		SynchronisedComponent {
			value,
            is_dirty: false
		}
	}

    pub(crate) fn get_and_clear_dity_bit(&mut self) -> bool {
        let is_dirty = self.is_dirty;
        self.is_dirty = false;
        is_dirty
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

impl<T: 'static + Sync + Send + Debug> Component for SynchronisedComponent<T>
where
    T: SpatialComponent,
{
    type Storage = VecStorage<Self>;
}


// pub type SpatialReadStorage<'a, T> = Storage<'a, SynchronisedComponent<T>, SpatialFetch<'a, MaskedStorage<SynchronisedComponent<T>>>>;
// pub type SpatialWriteStorage<'a, T> = WriteStorage<'a, SynchronisedComponent<T>>;


// pub struct SpatialReadStorage<'a, T>(ReadStorage<'a, SynchronisedComponent<T>>) where T: 'static + SpatialComponent + Sync + Send;
pub type SpatialReadStorage<'a, T> = ReadStorage<'a, SynchronisedComponent<T>>;
pub type SpatialWriteStorage<'a, T> = WriteStorage<'a, SynchronisedComponent<T>>;


// impl<'a, T> SystemData<'a> for SpatialReadStorage<'a, T>
// where
//     T: 'static + SpatialComponent + Sync + Send,
// {
//     fn setup(res: &mut Resources) {
//         ReadStorage::<'a, SynchronisedComponent<T>>::setup(res);
//     }

//     fn fetch(res: &'a Resources) -> Self {
//         // unimplemented!();
//         Storage::new(res.fetch(), res.fetch())
//         // SpatialReadStorage(ReadStorage::<'a, SynchronisedComponent<T>>::fetch(res))
//     }

//     fn reads() -> Vec<ResourceId> {
//         ReadStorage::<'a, SynchronisedComponent<T>>::reads()
//     }

//     fn writes() -> Vec<ResourceId> {
//         ReadStorage::<'a, SynchronisedComponent<T>>::writes()
//     }
// }



// pub struct SpatialFetch<'a, T: 'a>(Fetch<'a, T>);

// impl<'a, T> Deref for SpatialFetch<'a, T>
// where
//     T: Resource,
// {
//     type Target = T;

//     fn deref(&self) -> &T {
//         self.0.deref()
//     }
// }

// impl<'a, T> Clone for SpatialFetch<'a, T> {
//     fn clone(&self) -> Self {
//         SpatialFetch(Fetch {
//             inner: self.inner.clone(),
//             phantom: PhantomData,
//         })
//     }
// }

