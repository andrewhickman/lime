use std::iter;

use cassowary::strength::*;
use cassowary::WeightedRelation::*;
use cassowary::{Constraint, Variable};
use specs::prelude::*;

use super::{Constraints, Position};

pub struct Grid {
    rows: Vec<Variable>,
    cols: Vec<Variable>,
}

impl Component for Grid {
    type Storage = HashMapStorage<Self>;
}

impl Grid {
    pub fn new(pos: &Position, ncols: u32, nrows: u32) -> (Self, Constraints) {
        let rows = vars(pos.top(), nrows, pos.bottom());
        let cols = vars(pos.left(), ncols, pos.right());
        let cons = Constraints::new(cons(&rows).chain(cons(&cols)).collect());
        (Grid { rows, cols }, cons)
    }

    pub fn insert(&self, col: u32, row: u32, pos: &Position, cons: &mut Constraints) {
        cons.add(
            iter::empty()
                .chain(iter::once(
                    pos.left() | EQ(REQUIRED) | self.cols[col as usize],
                ))
                .chain(iter::once(
                    pos.top() | EQ(REQUIRED) | self.rows[row as usize],
                ))
                .chain(iter::once(
                    pos.right() | EQ(REQUIRED) | self.cols[(col + 1) as usize],
                ))
                .chain(iter::once(
                    pos.bottom() | EQ(REQUIRED) | self.rows[(row + 1) as usize],
                )),
        );
    }
}

fn vars(start: Variable, len: u32, end: Variable) -> Vec<Variable> {
    assert_ne!(len, 0);
    let mut vars = Vec::with_capacity((len + 1) as usize);
    vars.push(start);
    for _ in 0..(len - 1) {
        vars.push(Variable::new());
    }
    vars.push(end);
    vars
}

fn cons<'a>(vars: &'a [Variable]) -> impl Iterator<Item = Constraint> + 'a {
    let lt = vars.windows(2).map(|vs| vs[0] | LE(REQUIRED) | vs[1]);
    let sz = Variable::new();
    let eq = vars.windows(2).map(move |vs| vs[1] - vs[0] | EQ(WEAK) | sz);
    lt.chain(eq)
}
