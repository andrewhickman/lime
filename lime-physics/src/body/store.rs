use fnv::FnvHashMap;
use hibitset::BitSetLike;
use nphysics3d::object::BodyHandle;
use specs::prelude::*;
use specs::storage::UnprotectedStorage;
use specs::world::Index;

use body::Body;

#[derive(Default)]
pub struct BodyStorage {
    store: VecStorage<Body>,
    map: FnvHashMap<BodyHandle, Index>,
    removed: Vec<BodyHandle>,
}

impl BodyStorage {
    pub(crate) fn handle_removed<F>(store: &mut WriteStorage<Body>, handle: F)
    where
        F: FnOnce(&[BodyHandle]),
    {
        let removed = unsafe { &mut store.unprotected_storage_mut().removed };
        if !removed.is_empty() {
            handle(removed);
            removed.clear();
        }
    }
}

impl UnprotectedStorage<Body> for BodyStorage {
    unsafe fn clean<B>(&mut self, has: B)
    where
        B: BitSetLike,
    {
        self.store.clean(has)
    }

    unsafe fn get(&self, id: Index) -> &Body {
        self.store.get(id)
    }

    unsafe fn get_mut(&mut self, id: Index) -> &mut Body {
        self.store.get_mut(id)
    }

    unsafe fn insert(&mut self, id: Index, body: Body) {
        self.map.insert(body.handle, id);
        self.store.insert(id, body);
    }

    unsafe fn remove(&mut self, id: Index) -> Body {
        let body = self.store.remove(id);
        self.map.remove(&body.handle);
        self.removed.push(body.handle);
        body
    }
}
