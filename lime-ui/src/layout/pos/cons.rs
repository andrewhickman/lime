use cassowary::WeightedRelation::*;
use cassowary::{strength, Constraint};
use fnv::FnvHashSet;

use {Constraints, Position};

pub struct ConstraintsBuilder<'a> {
    pos: &'a Position,
    cons: FnvHashSet<Constraint>,
}

impl<'a> ConstraintsBuilder<'a> {
    pub(super) fn new(pos: &'a Position) -> Self {
        let mut cons = FnvHashSet::with_capacity_and_hasher(2, Default::default());
        cons.insert(pos.width() | GE(strength::REQUIRED) | 0.0);
        cons.insert(pos.width() | GE(strength::REQUIRED) | 0.0);
        ConstraintsBuilder { pos, cons }
    }

    pub fn with(mut self, con: Constraint) -> Self {
        self.cons.insert(con);
        self
    }

    pub fn center(mut self, other: &Position, strength: f64) -> Self {
        self.cons.insert(
            self.pos.left() - other.left() | EQ(strength) | other.right() - self.pos.right(),
        );
        self.cons.insert(
            self.pos.top() - other.top() | EQ(strength) | other.bottom() - self.pos.bottom(),
        );
        self
    }

    pub fn min_width(self, width: f64, strength: f64) -> Self {
        let con = self.pos.width() | GE(strength) | width;
        self.with(con)
    }

    pub fn min_height(self, height: f64, strength: f64) -> Self {
        let con = self.pos.height() | GE(strength) | height;
        self.with(con)
    }

    pub fn min_size(self, (width, height): (f64, f64), strength: f64) -> Self {
        self.min_width(width, strength).min_height(height, strength)
    }

    pub fn max_width(self, width: f64, strength: f64) -> Self {
        let con = self.pos.width() | LE(strength) | width;
        self.with(con)
    }

    pub fn max_height(self, height: f64, strength: f64) -> Self {
        let con = self.pos.height() | LE(strength) | height;
        self.with(con)
    }

    pub fn max_size(self, (width, height): (f64, f64), strength: f64) -> Self {
        self.max_width(width, strength).max_height(height, strength)
    }

    pub fn width(self, width: f64, strength: f64) -> Self {
        let con = self.pos.width() | EQ(strength) | width;
        self.with(con)
    }

    pub fn height(self, height: f64, strength: f64) -> Self {
        let con = self.pos.height() | EQ(strength) | height;
        self.with(con)
    }

    pub fn size(self, (width, height): (f64, f64), strength: f64) -> Self {
        self.width(width, strength).height(height, strength)
    }

    pub fn build(self) -> Constraints {
        Constraints::new(self.cons)
    }
}
