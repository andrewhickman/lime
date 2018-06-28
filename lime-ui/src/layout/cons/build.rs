use cassowary::WeightedRelation::*;
use cassowary::{strength, Constraint};
use fnv::FnvHashSet;

use layout::cons::ConstraintUpdate;
use layout::{Constraints, Position};

pub struct ConstraintsBuilder<'a> {
    pos: &'a Position,
    cons: FnvHashSet<Constraint>,
}

impl<'a> ConstraintsBuilder<'a> {
    pub(in layout) fn new(pos: &'a Position) -> Self {
        let mut cons = FnvHashSet::with_capacity_and_hasher(2, Default::default());
        cons.insert(pos.width_var() | GE(strength::REQUIRED) | 0.0);
        cons.insert(pos.height_var() | GE(strength::REQUIRED) | 0.0);
        ConstraintsBuilder { pos, cons }
    }

    pub fn with(mut self, con: Constraint) -> Self {
        self.cons.insert(con);
        self
    }

    pub fn center(mut self, other: &Position, strength: f64) -> Self {
        self.cons.insert(
            self.pos.left_var() - other.left_var()
                | EQ(strength)
                | other.right_var() - self.pos.right_var(),
        );
        self.cons.insert(
            self.pos.top_var() - other.top_var()
                | EQ(strength)
                | other.bottom_var() - self.pos.bottom_var(),
        );
        self
    }

    pub fn min_width(self, width: f64, strength: f64) -> Self {
        let con = self.pos.width_var() | GE(strength) | width;
        self.with(con)
    }

    pub fn min_height(self, height: f64, strength: f64) -> Self {
        let con = self.pos.height_var() | GE(strength) | height;
        self.with(con)
    }

    pub fn min_size(self, (width, height): (f64, f64), strength: f64) -> Self {
        self.min_width(width, strength).min_height(height, strength)
    }

    pub fn max_width(self, width: f64, strength: f64) -> Self {
        let con = self.pos.width_var() | LE(strength) | width;
        self.with(con)
    }

    pub fn max_height(self, height: f64, strength: f64) -> Self {
        let con = self.pos.height_var() | LE(strength) | height;
        self.with(con)
    }

    pub fn max_size(self, (width, height): (f64, f64), strength: f64) -> Self {
        self.max_width(width, strength).max_height(height, strength)
    }

    pub fn width(self, width: f64, strength: f64) -> Self {
        let con = self.pos.width_var() | EQ(strength) | width;
        self.with(con)
    }

    pub fn height(self, height: f64, strength: f64) -> Self {
        let con = self.pos.height_var() | EQ(strength) | height;
        self.with(con)
    }

    pub fn size(self, (width, height): (f64, f64), strength: f64) -> Self {
        self.width(width, strength).height(height, strength)
    }

    pub fn build(self) -> Constraints {
        let updates = self.cons
            .iter()
            .cloned()
            .map(ConstraintUpdate::Add)
            .collect();
        Constraints {
            cons: self.cons,
            updates,
            active: true,
        }
    }
}
