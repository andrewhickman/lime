use cassowary::{Constraint, Expression, Variable};
use cassowary::WeightedRelation::*;
use cassowary::strength::*;
use specs::prelude::*;

use super::{Constraints, Position};

pub struct Grid {
    rows: Vec<Variable>,
    cols: Vec<Variable>,
}

#[derive(Copy, Clone, Debug)]
pub enum Size {
    // Absolute size.
    Abs(f64),
    // Relative size.
    Rel(f64),
    // Space required is decided by children.
    Auto,
}

impl Component for Grid {
    type Storage = HashMapStorage<Self>;
}

impl Grid {
    pub fn new(
        pos: &Position,
        cols: impl IntoIterator<Item = Size>,
        rows: impl IntoIterator<Item = Size>,
    ) -> (Self, Constraints) {
        let rows = rows.into_iter();
        let cols = cols.into_iter();
        let mut cons = Vec::with_capacity(rows.size_hint().0 + cols.size_hint().0 + 4);
        let rows = layout(pos.top(), rows, pos.bottom(), &mut cons);
        let cols = layout(pos.left(), cols, pos.right(), &mut cons);
        let cons = Constraints::new(cons);
        (Grid { rows, cols }, cons)
    }

    pub fn insert(&self, col: u32, row: u32, pos: &Position, cons: &mut Constraints) {
        cons.add(vec![
            pos.left() | EQ(REQUIRED) | self.cols[col as usize],
            pos.top() | EQ(REQUIRED) | self.rows[row as usize],
            pos.right() | EQ(REQUIRED) |
                self.cols[(col + 1) as usize],
            pos.bottom() | EQ(REQUIRED) |
                self.rows[(row + 1) as usize],
        ]);
    }
}

fn layout(
    start: Variable,
    mid: impl Iterator<Item = Size>,
    end: Variable,
    cons: &mut Vec<Constraint>,
) -> Vec<Variable> {
    let mut vars = Vec::with_capacity(mid.size_hint().0 + 2);
    let mut size_sum = Expression::from_constant(0.0);
    let mut ratio_sum = 0.0;
    let rem = Variable::new();
    let mut flex_str = 0.0;

    vars.push(start);
    let mut prev = start;
    for size in mid {
        let var = Variable::new();
        vars.push(var);
        cons.push(prev | LE(REQUIRED) | var);

        // Tie-breaker constraint. First columns are filled first.
        cons.push(prev | EQ(flex_str) | var);
        flex_str += 0.001;

        match size {
            Size::Abs(size) => {
                cons.push(var - prev | EQ(STRONG) | size);
                size_sum = size_sum + var - prev;
            }
            Size::Rel(ratio) => {
                assert!(ratio > 0.0);
                cons.push(var - prev | EQ(STRONG) | ratio * rem);
                ratio_sum += ratio;
            }
            Size::Auto => {
                size_sum = size_sum + var - prev;
            }
        }

        prev = var;
    }

    let mult = ratio_sum.recip();
    if mult.is_normal() {
        cons.push(rem | EQ(REQUIRED) | (end - size_sum - start) * mult);
        cons.push(prev | EQ(REQUIRED) | end);
    } else {
        // No relative sizes. Use flex space.
        cons.push(start | EQ(flex_str) | prev);
        cons.push(prev | LE(REQUIRED) | end);
    }

    vars
}
