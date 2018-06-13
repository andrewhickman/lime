mod de;
mod style;
mod visibility;

pub use self::style::{Style, StyleEvent};
pub use self::visibility::{Visibility, VisibilityEvent, VisibilityState};

use render::d2::{Draw, Point};
use render::Color;
use specs::prelude::*;

use layout::Position;
use tree::{self, Node, Root};

#[derive(Clone, Component)]
pub enum Brush {
    Color(Color),
}

pub struct DrawUi;

type Data<'a> = (
    ReadExpect<'a, Root>,
    ReadStorage<'a, Node>,
    ReadStorage<'a, Brush>,
    ReadStorage<'a, Visibility>,
);

impl Draw for DrawUi {
    fn draw(&self, res: &Resources, visitor: &mut FnMut(&[Point], Color)) {
        let (root, nodes, brushes, viss) = Data::fetch(res);
        tree::walk(root.entity(), &nodes, |ent| {
            if viss.get(ent).map(Visibility::needs_draw).unwrap_or(true) {
                if let Some(brush) = brushes.get(ent) {
                    match *brush {
                        Brush::Color(color) => draw_color(ent, color, res, visitor),
                    }
                }
            }
        });
    }
}

fn draw_color(ent: Entity, color: Color, res: &Resources, visitor: &mut FnMut(&[Point], Color)) {
    let poss = ReadStorage::<Position>::fetch(res);
    if let Some(pos) = poss.get(ent) {
        visitor(&pos.tris(), color);
    }
}
