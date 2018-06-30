mod style;

pub use self::style::{Style, StyleEvent};

use render::d2::{Draw, Point};
use render::Color;
use specs::prelude::*;

use layout::Position;
use tree::{self, Node, Root, WalkPostResult, WalkPreResult};
use State;

#[derive(Clone, Component, Debug, Deserialize)]
pub enum Brush {
    Color(Color),
}

impl PartialEq for Brush {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Brush::Color(lhs), Brush::Color(rhs)) => lhs == rhs,
        }
    }
}

pub struct DrawUi;

type Data<'a> = (
    ReadExpect<'a, Root>,
    ReadStorage<'a, Node>,
    ReadStorage<'a, Brush>,
    ReadStorage<'a, State>,
);

impl Draw for DrawUi {
    fn draw(&self, res: &Resources, visitor: &mut FnMut(&[Point], Color)) {
        let (root, nodes, brushes, states) = Data::fetch(res);
        tree::walk::<(), _, _>(
            root.entity(),
            &nodes,
            &mut |ent| {
                if states.get(ent).map(State::needs_draw).unwrap_or(true) {
                    if let Some(brush) = brushes.get(ent) {
                        match *brush {
                            Brush::Color(color) => draw_color(ent, color, res, visitor),
                        }
                    }
                }
                WalkPreResult::Continue
            },
            &mut |_| WalkPostResult::Continue,
        );
    }
}

fn draw_color(ent: Entity, color: Color, res: &Resources, visitor: &mut FnMut(&[Point], Color)) {
    let poss = ReadStorage::<Position>::fetch(res);
    if let Some(pos) = poss.get(ent) {
        visitor(&pos.tris(), color);
    }
}
