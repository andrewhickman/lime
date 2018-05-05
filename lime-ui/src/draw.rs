use render::Color;
use render::d2::{Draw, Point};
use specs::prelude::*;

use {elem, ElementComponent};

pub struct DrawUi {
    root: Entity,
}

impl DrawUi {
    pub fn new(root: Entity) -> Self {
        DrawUi { root }
    }
}

impl Draw for DrawUi {
    fn draw(&self, res: &Resources, visitor: &mut FnMut(&[Point], Color)) {
        let elems = ReadStorage::<ElementComponent>::fetch(res);
        elem::visit_children(&elems, self.root, &mut |elem| elem.draw(res, visitor));
    }
}
