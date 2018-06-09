mod store;

pub use self::store::ConstraintsStorage;

use std::iter::FromIterator;
use std::mem;

use cassowary::Constraint;
use fnv::FnvHashSet;
use specs::prelude::*;

pub struct Constraints {
    cons: FnvHashSet<Constraint>,
    updates: Vec<ConstraintUpdate>,
    active: bool,
}

pub(in layout) enum ConstraintUpdate {
    Add(Constraint),
    Remove(Constraint),
}

impl Component for Constraints {
    type Storage = ConstraintsStorage;
}

impl Constraints {
    pub fn new(cons: FnvHashSet<Constraint>) -> Self {
        let updates = cons.iter().cloned().map(ConstraintUpdate::Add).collect();
        Constraints {
            cons,
            updates,
            active: true,
        }
    }

    pub fn add(&mut self, con: Constraint) {
        if self.cons.insert(con.clone()) && self.active {
            self.updates.push(ConstraintUpdate::Add(con));
        }
    }

    pub fn remove(&mut self, con: Constraint) {
        if self.cons.remove(&con) && self.active {
            self.updates.push(ConstraintUpdate::Remove(con));
        }
    }

    pub fn clear(&mut self) {
        self.updates
            .extend(self.cons.drain().map(ConstraintUpdate::Remove))
    }

    pub(in layout) fn expand(&mut self) {
        if !mem::replace(&mut self.active, false) {
            self.updates
                .extend(self.cons.iter().cloned().map(ConstraintUpdate::Add))
        }
    }

    pub(in layout) fn collapse(&mut self) {
        if mem::replace(&mut self.active, false) {
            self.updates
                .extend(self.cons.iter().cloned().map(ConstraintUpdate::Remove))
        }
    }
}

impl FromIterator<Constraint> for Constraints {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Constraint>,
    {
        Constraints::new(FnvHashSet::from_iter(iter))
    }
}
