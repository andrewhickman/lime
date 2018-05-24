use render::d2::{Draw, Point};
use render::Color;
use specs::prelude::*;

use {tree, Node, Position, Root};

pub enum Brush {
    Color(Color),
}

impl Component for Brush {
    type Storage = DenseVecStorage<Self>;
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
        tree::walk(root.entity(), &nodes, &mut |ent| {
            if let Some((pos, brush)) = join.get(ent, &ents) {
                match *brush {
                    Brush::Color(color) => visitor(&pos.tris(), color),
                }
            }
        });
    }
}
