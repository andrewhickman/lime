use render::Color;
use render::d2::{Draw, Point};
use specs::prelude::*;

use Element;

pub struct Rect {
    tl: Point,
    br: Point,
    color: Color,
}

impl Rect {
    pub fn new(tl: Point, br: Point, color: Color) -> Self {
        Rect { tl, br, color }
    }
}

impl Element for Rect {}

impl Draw for Rect {
    fn draw(&self, _: &Resources, visitor: &mut FnMut(&[Point], Color)) {
        let (tl, br) = (self.br, self.tl);
        let vertices = [Point(tl.0, br.1), tl, br, br, tl, Point(br.0, tl.1)];
        visitor(&vertices, self.color)
    }
}
