use std::iter;

use cassowary::strength::*;
use cassowary::WeightedRelation::*;
use cassowary::{Constraint, Expression, Variable};
use specs::prelude::*;

/// Represents the outermost bounding rect of a ui element.
#[derive(Copy, Clone, Component, Debug)]
pub struct PositionVars {
    pub left: Variable,
    pub top: Variable,
    pub right: Variable,
    pub bottom: Variable,
}

impl PositionVars {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn width(&self) -> Expression {
        self.right - self.left
    }

    pub fn height(&self) -> Expression {
        self.bottom - self.top
    }

    pub fn base(&self) -> impl Iterator<Item = Constraint> {
        iter::empty()
            .chain(iter::once(self.left | LE(REQUIRED) | self.right))
            .chain(iter::once(self.top | LE(REQUIRED) | self.bottom))
    }

    pub fn min_width(&self, width: f64, strength: f64) -> impl Iterator<Item = Constraint> {
        iter::once(self.width() | GE(strength) | width)
    }

    pub fn min_height(&self, width: f64, strength: f64) -> impl Iterator<Item = Constraint> {
        iter::once(self.height() | GE(strength) | width)
    }

    pub fn min_size(&self, (w, h): (f64, f64), strength: f64) -> impl Iterator<Item = Constraint> {
        iter::empty()
            .chain(self.min_width(w, strength))
            .chain(self.min_height(h, strength))
    }
}

impl Default for PositionVars {
    fn default() -> Self {
        PositionVars {
            left: Variable::new(),
            top: Variable::new(),
            right: Variable::new(),
            bottom: Variable::new(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ScreenVars {
    pub width: Variable,
    pub height: Variable,
}

impl ScreenVars {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Default for ScreenVars {
    fn default() -> Self {
        ScreenVars {
            width: Variable::new(),
            height: Variable::new(),
        }
    }
}
