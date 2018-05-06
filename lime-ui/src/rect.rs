use cassowary::Solver;
use render::d2::{Draw, Point};
use render::Color;
use specs::prelude::*;

use layout::{LayoutVars, Resize};
use {Element, Layout};

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

impl Layout for Rect {
    fn add_constraints(&self, _: &ReadStorage<LayoutVars>, this: &LayoutVars, parent: &LayoutVars, solver: &mut Solver) {
        use cassowary::{strength::*, WeightedRelation::*};

        solver.add_constraint(this.bottom |EQ(REQUIRED)| parent.bottom).unwrap();
        solver.add_constraint(this.top |EQ(REQUIRED)| parent.top).unwrap();
    }

    fn resize(&mut self, resize: &Resize) {
        println!("resize: {:?}", resize);
        fn update(old: &mut f32, new: Option<f64>) {
            if let Some(new) = new {
                *old = new as f32;
            }
        }

        update(&mut self.tl.0, resize.left);
        update(&mut self.tl.1, resize.top);
        update(&mut self.br.0, resize.right);
        update(&mut self.br.1, resize.bottom);
    }
}
