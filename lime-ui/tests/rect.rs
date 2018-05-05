extern crate lime_render as render;
extern crate lime_ui as ui;
extern crate specs;
extern crate winit;

use render::{d2::Point, d3, Renderer, Color};
use ui::{ElementComponent, Rect, DrawUi};
use specs::prelude::*;
use winit::{Event, EventsLoop, WindowBuilder, WindowEvent};

pub struct D3;

impl d3::Draw for D3 {
    fn draw(&self, _: &Resources, _: &mut FnMut(&d3::Mesh, Color)) {}
}

#[test]
fn rect() {
    let mut events_loop = EventsLoop::new();
    let builder = WindowBuilder::new();
    let mut renderer = Renderer::new(&events_loop, builder);
    let mut world = World::new();
    world.register::<ElementComponent>();
    let root = world.create_entity()
        .with(Box::new(Rect::new(Point(100.0, 100.0), Point(500.0, 300.0), Color::RED)) as ElementComponent)
        .build();
    world.add_bundle(renderer.bundle(D3, DrawUi::new(root)));

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
