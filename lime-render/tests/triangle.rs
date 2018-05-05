extern crate lime_render as render;
extern crate specs;
extern crate winit;

use render::{Color, Renderer, d2, d3};
use specs::prelude::*;
use winit::{Event, EventsLoop, WindowBuilder, WindowEvent};

struct D3;

impl d3::Draw for D3 {
    fn draw(&self, _: &Resources, _: &mut FnMut(&d3::Mesh, Color)) {}
}

struct D2;

impl d2::Draw for D2 {
    fn draw(&self, _: &Resources, visitor: &mut FnMut(&[d2::Point], Color)) {
        static VERTICES: [d2::Point; 3] = [
            d2::Point(100.0, 100.0),
            d2::Point(200.0, 100.0),
            d2::Point(100.0, 200.0),
        ];
        visitor(&VERTICES, Color::RED)
    }
}

#[test]
fn triangle() {
    let mut events_loop = EventsLoop::new();
    let builder = WindowBuilder::new();
    let mut renderer = Renderer::new(&events_loop, builder);
    let mut world = World::new();
    world.add_bundle(renderer.bundle(D3, D2));

    let mut quit = false;
    while !quit {
        events_loop.poll_events(|event| {
            match event {
                Event::WindowEvent {
                    event: WindowEvent::Closed,
                    ..
                } => quit = true,
                _ => (),
            };
        });

        renderer.run_now(&mut world.res);
    }
}
