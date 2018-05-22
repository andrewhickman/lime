use std::iter::{self, FromIterator};
use std::ops::{Deref, Range};

use cassowary::Constraint;
use chan;
use specs::prelude::*;

#[derive(Component, Debug)]
pub struct Constraints {
    cons: Vec<Constraint>,
    tx: chan::Sender<ConstraintUpdate>,
}

#[derive(Clone, Debug)]
pub enum ConstraintUpdate {
    Add(Constraint),
    Remove(Constraint),
}

impl Constraints {
    pub fn new(tx: chan::Sender<ConstraintUpdate>) -> Self {
        Constraints {
            cons: Vec::new(),
            tx,
        }
    }

    pub fn from_iter<I>(tx: chan::Sender<ConstraintUpdate>, iter: I) -> Self
    where
        I: IntoIterator<Item = Constraint>,
    {
        let cons = Vec::from_iter(iter);
        for con in &cons {
            tx.send(ConstraintUpdate::Add(con.clone()));
        }

        Constraints { cons, tx }
    }

    pub fn insert(&mut self, con: Constraint) {
        self.extend(iter::once(con));
    }

    pub fn remove(&mut self, idx: usize) {
        self.tx
            .send(ConstraintUpdate::Remove(self.cons.remove(idx)))
    }

    pub fn remove_range(&mut self, range: Range<usize>) {
        for con in self.cons.drain(range) {
            self.tx.send(ConstraintUpdate::Remove(con))
        }
    }
}

impl Extend<Constraint> for Constraints {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = Constraint>,
    {
        let Constraints { cons, tx } = self;
        cons.extend(
            iter.into_iter()
                .inspect(|con| tx.send(ConstraintUpdate::Add(con.clone()))),
        )
    }
}

impl Deref for Constraints {
    type Target = [Constraint];

    fn deref(&self) -> &Self::Target {
        &self.cons
    }
}

impl Drop for Constraints {
    fn drop(&mut self) {
        for con in self.cons.drain(..) {
            self.tx.send(ConstraintUpdate::Remove(con));
        }
    }
}
