mod cons;

pub use self::cons::ConstraintsBuilder;

use cassowary::{Expression, Variable};
use fnv::FnvHashMap;
use render::d2::Point;
use specs::prelude::*;

#[derive(Debug)]
pub struct Position {
    // Order: left, top, right, bottom
    vars: [Variable; 4],
    vals: [f32; 4],
}

impl Component for Position {
    type Storage = DenseVecStorage<Self>;
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

    pub fn bl(&self) -> Point {
        Point(self.vals[0], self.vals[3])
    }

    pub fn tl(&self) -> Point {
        Point(self.vals[0], self.vals[1])
    }

    pub fn br(&self) -> Point {
        Point(self.vals[2], self.vals[3])
    }

    pub fn tr(&self) -> Point {
        Point(self.vals[2], self.vals[1])
    }

    pub fn left(&self) -> Variable {
        self.vars[0]
    }

    pub fn top(&self) -> Variable {
        self.vars[1]
    }

    pub fn right(&self) -> Variable {
        self.vars[2]
    }

    pub fn bottom(&self) -> Variable {
        self.vars[3]
    }

    pub fn width(&self) -> Expression {
        self.right() - self.left()
    }

    pub fn height(&self) -> Expression {
        self.bottom() - self.top()
    }

    pub fn tris(&self) -> [Point; 6] {
        [
            self.bl(),
            self.tl(),
            self.br(),
            self.br(),
            self.tl(),
            self.tr(),
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
