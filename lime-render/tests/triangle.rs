extern crate lime_render as render;
extern crate specs;
extern crate winit;

mod common;

use render::Renderer;
use specs::prelude::*;
use winit::{Event, EventsLoop, WindowBuilder, WindowEvent};

#[test]
fn triangle() {
    let mut events_loop = EventsLoop::new();
    let builder = WindowBuilder::new();
    let mut renderer = Renderer::new(&events_loop, builder);
    let mut world = World::new();
    world.add_bundle(renderer.bundle(common::D3, common::D2));

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
