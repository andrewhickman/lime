extern crate env_logger;
extern crate lime_render as render;
extern crate shrev;
extern crate specs;
extern crate winit;

use render::{d2, Color};
use shrev::EventChannel;
use specs::prelude::*;
use winit::{Event, EventsLoop, WindowBuilder, WindowEvent};

struct D3;

impl<'a> System<'a> for D3 {
    type SystemData = ();

    fn run(&mut self, (): Self::SystemData) {}
}

struct D2;

impl<'a> System<'a> for D2 {
    type SystemData = WriteExpect<'a, d2::Renderer>;

    fn run(&mut self, mut renderer: Self::SystemData) {
        static VERTICES: [d2::Point; 3] = [
            d2::Point(100.0, 100.0),
            d2::Point(200.0, 100.0),
            d2::Point(100.0, 200.0),
        ];
        renderer.draw_tri(&VERTICES, Color::RED)
    }
}

fn main() {
    env_logger::init();

    let mut events_loop = EventsLoop::new();
    let window = WindowBuilder::new().build(&events_loop).unwrap();
    let mut world = World::new();
    let mut dispatcher = DispatcherBuilder::new()
        .with(D3, "D3", &[])
        .with(D2, "D2", &[]);
    world.add_resource(EventChannel::<winit::Event>::new());
    render::init(&mut world, &mut dispatcher, window, "D3", "D2");
    let mut dispatcher = dispatcher.build();

    let mut quit = false;
    while !quit {
        events_loop.poll_events(|event| {
            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => quit = true,
                event => world
                    .write_resource::<EventChannel<Event>>()
                    .single_write(event),
            };
        });

        dispatcher.run_now(&mut world.res);
    }
}
