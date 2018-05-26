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
use specs::prelude::*;
use ui::{Brush, Constraints, DrawUi, Node, Position, Root};
use winit::{Event, EventsLoop, WindowBuilder, WindowEvent};

use common::D3;

fn main() {
    env_logger::init();
    std::panic::set_hook(Box::new(utils::panic_hook));

    let mut events_loop = EventsLoop::new();
    let builder = WindowBuilder::new();
    let mut world = World::new();
    let renderer = render::init(&mut world, &events_loop, builder, D3, DrawUi);
    let layout_sys = ui::init(&mut world);

    let mut dispatcher = DispatcherBuilder::new()
        .with_thread_local(layout_sys)
        .with_thread_local(renderer)
        .build();

    let root = world.read_resource::<Root>().entity();

    let pos = Position::new();
    let cons = {
        let poss = world.read_storage::<Position>();
        Constraints::new(
            pos.min_size((200.0, 400.0), STRONG)
                .chain(pos.center(poss.get(root).unwrap(), REQUIRED))
                .collect(),
        )
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
                Event::WindowEvent { event: WindowEvent::Closed, .. } => quit = true,
                _ => (),
            };
        });

        dispatcher.dispatch(&world.res);
    }
}
