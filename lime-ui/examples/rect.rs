extern crate cassowary;
extern crate env_logger;
extern crate lime_render as render;
extern crate lime_ui as ui;
extern crate lime_utils as utils;
extern crate specs;
extern crate winit;

mod common;

use cassowary::strength::*;
use render::Color;
use ui::draw::Brush;
use ui::layout::Position;
use ui::tree::{Node, Root};
use winit::{Event, EventsLoop, WindowEvent};

fn main() {
    let mut events_loop = EventsLoop::new();
    let (mut world, mut dispatcher) = common::init(&events_loop);

    let root = world.read_resource::<Root>().entity();

    let pos = Position::new();
    let cons = {
        let poss = world.read_storage::<Position>();
        pos.constraints_builder()
            .size((400.0, 200.0), STRONG)
            .center(poss.get(root).unwrap(), STRONG)
            .build()
    };

    Node::with_parent(world.create_entity(), root)
        .with(pos)
        .with(cons)
        .with(Brush::Color(Color::RED))
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

        dispatcher.dispatch(&world.res);
    }
}
