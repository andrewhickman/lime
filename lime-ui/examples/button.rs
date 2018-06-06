extern crate cassowary;
extern crate env_logger;
extern crate lime_render as render;
extern crate lime_ui as ui;
extern crate lime_utils as utils;
extern crate specs;
extern crate winit;

mod common;

use std::sync::Arc;

use cassowary::strength::*;
use render::Color;
use ui::draw::{Brush, StyleDef, Style};
use ui::tree::{Node, Root};
use ui::layout::Position;
use ui::event::{EventSystem, Button};
use specs::prelude::*;
use winit::{Event, EventsLoop, WindowEvent};

fn main() {
    env_logger::init();
    std::panic::set_hook(Box::new(utils::panic_hook));

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

    let style_def = Arc::new(StyleDef {
        btn_disabled: Brush::Color(Color::new(0.2, 0.2, 0.2, 1.0)),
        btn_normal: Brush::Color(Color::RED),
        btn_focused: Brush::Color(Color::GREEN),
        btn_pressed: Brush::Color(Color::BLUE),
    });

    let style = Style::new(style_def.clone());

    Node::with_parent(world.create_entity(), root)
        .with(pos)
        .with(cons)
        .with(Button::new(true))
        .with(style)
        .with(style_def.btn_normal.clone())
        .build();

    let mut quit = false;
    while !quit {
        events_loop.poll_events(|event| {
            match event {
                Event::WindowEvent {
                    event: WindowEvent::Closed,
                    ..
                } => quit = true,
                ev => EventSystem(&ev).run_now(&world.res),
            };
        });

        dispatcher.dispatch(&world.res);
    }
}
