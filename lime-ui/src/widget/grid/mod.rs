pub(crate) mod de;
#[cfg(test)]
mod tests;

use cassowary::strength::*;
use cassowary::WeightedRelation::*;
use cassowary::{Expression, Variable};
use specs::prelude::*;

use layout::{Constraints, Position};

#[derive(Component)]
#[storage(HashMapStorage)]
pub struct Grid {
    rows: Vec<Variable>,
    cols: Vec<Variable>,
}

#[derive(Copy, Clone, Debug, Deserialize, PartialEq)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum Size {
    // Absolute size.
    Abs(f64),
    // Relative size.
    Rel(f64),
    // Space required is decided by children.
    Auto,
}

impl Grid {
    pub fn new(
        pos: &Position,
        cons: &mut Constraints,
        cols: impl IntoIterator<Item = Size>,
        rows: impl IntoIterator<Item = Size>,
    ) -> Self {
        let rows = rows.into_iter();
        let cols = cols.into_iter();
        cons.reserve(2 * rows.size_hint().0 + 2 * cols.size_hint().0 + 4);
        let rows = layout(pos.top_var(), rows, pos.bottom_var(), cons);
        let cols = layout(pos.left_var(), cols, pos.right_var(), cons);
        Grid { rows, cols }
    }

    pub fn insert(&self, col: u32, row: u32, pos: &Position, cons: &mut Constraints) {
        self.insert_col(col, pos, cons);
        self.insert_row(row, pos, cons);
    }

    pub fn insert_col(&self, col: u32, pos: &Position, cons: &mut Constraints) {
        cons.add(pos.left_var() | EQ(REQUIRED) | self.cols[col as usize]);
        cons.add(pos.right_var() | EQ(REQUIRED) | self.cols[(col + 1) as usize]);
    }

    pub fn insert_row(&self, row: u32, pos: &Position, cons: &mut Constraints) {
        cons.add(pos.top_var() | EQ(REQUIRED) | self.rows[row as usize]);
        cons.add(pos.bottom_var() | EQ(REQUIRED) | self.rows[(row + 1) as usize]);
    }
}

fn layout(
    start: Variable,
    mid: impl Iterator<Item = Size>,
    end: Variable,
    cons: &mut Constraints,
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
        cons.add(prev | LE(REQUIRED) | var);

        // Tie-breaker constraint. First columns are filled first.
        cons.add(prev | EQ(flex_str) | var);
        flex_str += 0.001;

        match size {
            Size::Abs(size) => {
                cons.add(var - prev | EQ(STRONG) | size);
                size_sum += var - prev;
            }
            Size::Rel(ratio) => {
                assert!(ratio > 0.0);
                cons.add(var - prev | EQ(STRONG) | ratio * rem);
                ratio_sum += ratio;
            }
            Size::Auto => {
                size_sum += var - prev;
            }
        }

        prev = var;
    }

    let mult = ratio_sum.recip();
    if mult.is_normal() {
        cons.add(rem | EQ(REQUIRED) | (end - size_sum - start) * mult);
        cons.add(prev | EQ(REQUIRED) | end);
    } else {
        // No relative sizes. Use flex space.
        cons.add(start | EQ(flex_str) | prev);
        cons.add(prev | LE(REQUIRED) | end);
    }

    vars
}
