mod visibility;

pub use self::visibility::{Visibility, VisibilityEvent, VisibilityState};

use std::sync::Arc;

use render::d2::{Draw, Point};
use render::Color;
use specs::prelude::*;

use layout::Position;
use tree::{self, Node, Root};

pub trait Style: Send + Sync + 'static {
    fn draw(&self, ent: Entity, res: &Resources, visitor: &mut FnMut(&[Point], Color));
}

#[derive(Clone, Component)]
pub enum Brush {
    Color(Color),
    Style(Arc<Style>),
}

pub struct DrawUi;

impl Style for Brush {
    fn draw(&self, ent: Entity, res: &Resources, visitor: &mut FnMut(&[Point], Color)) {
        match *self {
            Brush::Color(color) => draw_color(ent, color, res, visitor),
            Brush::Style(ref style) => style.draw(ent, res, visitor),
        }
    }
}

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
                    brush.draw(ent, res, visitor);
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
