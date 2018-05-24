use specs::prelude::*;
use render::Color;
use render::d2::{Draw, Point};

use {tree, Root, Position, Node};

#[derive(Component)]
pub enum Brush {
    Color(Color),
}

pub struct DrawUi;

type Data<'a> = (
    ReadExpect<'a, Root>,
    Entities<'a>,
    ReadStorage<'a, Node>,
    ReadStorage<'a, Position>,
    ReadStorage<'a, Brush>,
);

impl Draw for DrawUi {
    fn draw(&self, res: &Resources, visitor: &mut FnMut(&[Point], Color)) {
        let (root, ents, nodes, poss, brushes) = Data::fetch(res);
        let mut join = (&poss, &brushes).join();
        tree::walk(root.entity(), &nodes, &mut |ent| if let Some((pos, brush)) = join.get(ent, &ents) {
            match *brush {
                Brush::Color(color) => visitor(&pos.tris(), color),
            }
        });
    }
}
