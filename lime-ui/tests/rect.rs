extern crate lime_render as render;
extern crate lime_ui as ui;
extern crate specs;
extern crate winit;

use render::{d2::Point, d3, Color, Renderer};
use specs::prelude::*;
use ui::{elem, DrawUi, ElementComponent, Rect, LayoutSystem};
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
    world.add_bundle(renderer.bundle(D3, DrawUi));
    world.add_bundle(ui::Bundle::new());

    elem::add_root(&mut world, Rect::new(
        Point(100.0, 100.0),
        Point(500.0, 300.0),
        Color::RED,
    ));

    let mut dispatcher = DispatcherBuilder::new()
        .with_thread_local(LayoutSystem::new(&world.res))
        .build();

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
        dispatcher.dispatch(&world.res);
    }
}
