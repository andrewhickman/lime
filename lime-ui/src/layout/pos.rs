use cassowary::{Expression, Variable};
use fnv::FnvHashMap;
use render::d2::Point;
use specs::prelude::*;

use layout::ConstraintsBuilder;

#[derive(Clone, Component, Debug)]
pub struct Position {
    // Order: left, top, right, bottom
    vars: [Variable; 4],
    vals: [f32; 4],
}

impl Default for Position {
    fn default() -> Self {
        Position::new()
    }
}

impl Position {
    pub(in layout) fn root(zero: Variable, width: Variable, height: Variable) -> Self {
        Position {
            vars: [zero, zero, width, height],
            vals: [0.0; 4],
        }
    }

    pub fn new() -> Self {
        Position {
            vars: [
                Variable::new(),
                Variable::new(),
                Variable::new(),
                Variable::new(),
            ],
            vals: [0.0; 4],
        }
    }

    pub fn left(&self) -> f32 {
        self.vals[0]
    }

    pub fn top(&self) -> f32 {
        self.vals[1]
    }

    pub fn right(&self) -> f32 {
        self.vals[2]
    }

    pub fn bottom(&self) -> f32 {
        self.vals[3]
    }

    pub fn width(&self) -> f32 {
        self.right() - self.left()
    }

    pub fn height(&self) -> f32 {
        self.bottom() - self.top()
    }

    pub fn bottom_left(&self) -> Point {
        Point(self.vals[0], self.vals[3])
    }

    pub fn top_left(&self) -> Point {
        Point(self.vals[0], self.vals[1])
    }

    pub fn bottom_right(&self) -> Point {
        Point(self.vals[2], self.vals[3])
    }

    pub fn top_right(&self) -> Point {
        Point(self.vals[2], self.vals[1])
    }

    pub fn left_var(&self) -> Variable {
        self.vars[0]
    }

    pub fn top_var(&self) -> Variable {
        self.vars[1]
    }

    pub fn right_var(&self) -> Variable {
        self.vars[2]
    }

    pub fn bottom_var(&self) -> Variable {
        self.vars[3]
    }

    pub fn width_var(&self) -> Expression {
        self.right_var() - self.left_var()
    }

    pub fn height_var(&self) -> Expression {
        self.bottom_var() - self.top_var()
    }

    pub fn tris(&self) -> [Point; 6] {
        [
            self.bottom_left(),
            self.top_left(),
            self.bottom_right(),
            self.bottom_right(),
            self.top_left(),
            self.top_right(),
        ]
    }

    pub fn contains(&self, point: Point) -> bool {
        self.vals[0] <= point.0
            && point.0 < self.vals[2]
            && self.vals[1] <= point.1
            && point.1 < self.vals[3]
    }

    pub fn constraints_builder(&self) -> ConstraintsBuilder {
        ConstraintsBuilder::new(self)
    }

    pub(in layout) fn update(&mut self, changes: &FnvHashMap<Variable, f64>) {
        for i in 0..4 {
            if let Some(&val) = changes.get(&self.vars[i]) {
                self.vals[i] = val as f32;
            }
        }
    }
}
