extern crate env_logger;
extern crate image;
extern crate lime_render as render;
extern crate shrev;
extern crate specs;
extern crate winit;

mod common;

use std::path::Path;

use render::{d2, Color};
use specs::prelude::*;

struct D3;

impl<'a> System<'a> for D3 {
    type SystemData = ();

    fn run(&mut self, (): Self::SystemData) {}
}

fn test_d2(name: impl AsRef<Path>, dims: [u32; 2], f: impl FnMut(&mut d2::Renderer) + Send) {
    struct D2<F>(F);

    impl<'a, F> System<'a> for D2<F>
    where
        F: FnMut(&mut d2::Renderer) + Send,
    {
        type SystemData = WriteExpect<'a, d2::Renderer>;

        fn run(&mut self, mut r: Self::SystemData) {
            (self.0)(&mut r)
        }
    }

    common::test(Path::new("d2").join(name), D3, D2(f), dims);
}

#[test]
fn triangle() {
    test_d2("triangle", [500, 500], |r| {
        static VERTICES: [d2::Point; 3] = [
            d2::Point(100.0, 100.0),
            d2::Point(200.0, 100.0),
            d2::Point(100.0, 200.0),
        ];
        r.draw_tris(&VERTICES, Color::RED)
    })
}
