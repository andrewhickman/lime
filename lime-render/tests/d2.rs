extern crate env_logger;
extern crate image;
extern crate lime_render as render;
extern crate shrev;
extern crate specs;
extern crate winit;

mod common;

use std::f32::consts::PI;
use std::path::Path;

use render::d2::{Point, Renderer};
use render::Color;
use specs::prelude::*;

struct D3;

impl<'a> System<'a> for D3 {
    type SystemData = ();

    fn run(&mut self, (): Self::SystemData) {}
}

fn test_d2(name: impl AsRef<Path>, dims: [u32; 2], f: impl FnMut(&mut Renderer) + Send) {
    struct D2<F>(F);

    impl<'a, F> System<'a> for D2<F>
    where
        F: FnMut(&mut Renderer) + Send,
    {
        type SystemData = WriteExpect<'a, Renderer>;

        fn run(&mut self, mut r: Self::SystemData) {
            (self.0)(&mut r)
        }
    }

    common::test(Path::new("d2").join(name), D3, D2(f), dims);
}

#[test]
fn triangle() {
    test_d2("triangle", [500, 500], |r| {
        r.draw_tris(
            &[
                Point(100.0, 100.0),
                Point(200.0, 100.0),
                Point(100.0, 200.0),
            ],
            Color::RED,
        )
    })
}

#[test]
fn circle() {
    const N: usize = 17;

    let mut vertices = Vec::new();
    let mut prev = Point(450.0, 250.0);
    for i in 1..=N {
        let (sin, cos) = (2.0 * i as f32 * PI / N as f32).sin_cos();
        vertices.push(Point(250.0, 250.0));
        vertices.push(prev);
        prev = Point(250.0 + cos * 200.0, 250.0 + sin * 200.0);
        vertices.push(prev);
    }

    test_d2("circle", [500, 500], |r| {
        r.draw_tris(&vertices, Color::BLUE)
    })
}
