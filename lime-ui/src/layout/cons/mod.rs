mod build;
mod store;

pub use self::build::ConstraintsBuilder;
pub use self::store::ConstraintsStorage;

use std::mem;

use cassowary::Constraint;
use fnv::FnvHashSet;
use specs::prelude::*;

use layout::Position;

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
    pub fn new(pos: &Position) -> Self {
        ConstraintsBuilder::new(pos).build()
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

    pub fn reserve(&mut self, cap: usize) {
        self.cons.reserve(cap);
        if self.active {
            self.updates.reserve(cap);
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

impl Extend<Constraint> for Constraints {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = Constraint>,
    {
        for con in iter {
            self.add(con)
        }
    }
}
