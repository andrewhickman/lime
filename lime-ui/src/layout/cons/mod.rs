mod store;

pub use self::store::ConstraintStorage;

use std::ops::Range;

use cassowary::Constraint;
use specs::prelude::*;

pub struct Constraints {
    cons: Vec<Constraint>,
    updates: Vec<ConstraintUpdate>,
}

impl Component for Constraints {
    type Storage = ConstraintStorage;
}

impl Constraints {
    pub fn new(cons: Vec<Constraint>) -> Self {
        let updates = cons.iter().cloned().map(ConstraintUpdate::Add).collect();
        Constraints { cons, updates }
    }

    pub fn add(&mut self, iter: impl IntoIterator<Item = Constraint>) -> Range<usize> {
        let old_len = self.cons.len();
        self.cons.extend(iter);
        let range = old_len..self.cons.len();
        self.updates.extend(
            self.cons[range.clone()]
                .iter()
                .cloned()
                .map(ConstraintUpdate::Add),
        );
        range
    }

    pub fn remove<'a>(&'a mut self, range: Range<usize>) -> impl Iterator<Item = Constraint> + 'a {
        self.updates.extend(
            self.cons[range.clone()]
                .iter()
                .cloned()
                .map(ConstraintUpdate::Remove),
        );
        self.cons.drain(range)
    }

    pub fn clear(&mut self) {
        self.updates
            .extend(self.cons.drain(..).map(ConstraintUpdate::Remove))
    }

    pub(in layout) fn expand(&mut self) {
        self.updates
            .extend(self.cons.iter().cloned().map(ConstraintUpdate::Add))
    }

    pub(in layout) fn collapse(&mut self) {
        self.updates
            .extend(self.cons.iter().cloned().map(ConstraintUpdate::Remove))
    }
}

pub(in layout) enum ConstraintUpdate {
    Add(Constraint),
    Remove(Constraint),
}
