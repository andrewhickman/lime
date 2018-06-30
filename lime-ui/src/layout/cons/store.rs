use hibitset::BitSetLike;
use specs::prelude::*;
use specs::storage::UnprotectedStorage;
use specs::world::Index;

use super::{ConstraintUpdate, Constraints};

#[derive(Default)]
pub struct ConstraintsStorage {
    store: DenseVecStorage<Constraints>,
    // Updates flushed on removal of Constraints.
    updates: Vec<ConstraintUpdate>,
}

impl ConstraintsStorage {
    pub(in layout) fn handle_updates<F>(store: &mut WriteStorage<Constraints>, mut handle: F)
    where
        F: FnMut(ConstraintUpdate),
    {
        for con in store.join() {
            con.updates.drain(..).for_each(&mut handle);
        }

        unsafe { store.unprotected_storage_mut() }
            .updates
            .drain(..)
            .for_each(&mut handle);
    }
}

impl UnprotectedStorage<Constraints> for ConstraintsStorage {
    unsafe fn clean<B>(&mut self, has: B)
    where
        B: BitSetLike,
    {
        self.store.clean(has)
    }

    unsafe fn get(&self, id: Index) -> &Constraints {
        self.store.get(id)
    }

    unsafe fn get_mut(&mut self, id: Index) -> &mut Constraints {
        self.store.get_mut(id)
    }

    unsafe fn insert(&mut self, id: Index, cons: Constraints) {
        self.store.insert(id, cons);
    }

    unsafe fn remove(&mut self, id: Index) -> Constraints {
        let mut cons = self.store.remove(id);
        if cons.active {
            cons.clear();
            self.updates.append(&mut cons.updates);
        }
        cons
    }
}
