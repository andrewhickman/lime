use render::d2::{Draw, Point};
use render::Color;
use specs::prelude::*;

use {elem, Element, ElementComponent};

pub struct DrawUi;

type DrawData<'a> = (
    ReadExpect<'a, elem::Root>,
    ReadStorage<'a, ElementComponent>,
);

impl Draw for DrawUi {
    fn draw(&self, res: &Resources, visitor: &mut FnMut(&[Point], Color)) {
        let (root, elems) = DrawData::fetch(res);
        for &elem in &root.stack {
            visit_children(&elems, elem, &mut |e| e.draw(res, visitor));
        }
    }
}

fn visit_children<F>(store: &ReadStorage<ElementComponent>, elem: Entity, visitor: &mut F)
where
    F: FnMut(&Element),
{
    let elem: &Element = store.get(elem).unwrap();
    visitor(elem);
    for &child in elem.children().iter() {
        visit_children(store, child, visitor)
    }
}
