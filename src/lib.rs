use spatialos_sdk::worker::component::Component as SpatialComponent;
use std::ops::Deref;
use specs::prelude::*;
use spatialos_sdk::worker::op::*;
use specs::shred::{Fetch, ResourceId, SystemData, Resource};
use specs::storage::MaskedStorage;
use std::marker::PhantomData;

pub mod world;

pub struct SynchronisedComponent<T: SpatialComponent> {
    pub value: T,
}

impl<T: SpatialComponent> SynchronisedComponent<T> {
	pub fn new(value: T) -> SynchronisedComponent<T> {
		SynchronisedComponent {
			value
		}
	}
}

impl<T: SpatialComponent> Deref for SynchronisedComponent<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T: 'static + Sync + Send> Component for SynchronisedComponent<T>
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