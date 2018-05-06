use render::d2::{Draw, Point};
use render::Color;
use specs::prelude::*;

use {elem, ElementComponent};

pub struct DrawUi;

type DrawData<'a> = (
    ReadExpect<'a, elem::Root>,
    ReadStorage<'a, ElementComponent>,
);

impl Draw for DrawUi {
    fn draw(&self, res: &Resources, visitor: &mut FnMut(&[Point], Color)) {
        let (root, elems) = DrawData::fetch(res);
        for &elem in &root.stack {
            elem::visit_children(&elems, elem, &mut |e| e.draw(res, visitor));
        }
    }
}
