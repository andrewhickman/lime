extern crate cassowary;
extern crate env_logger;
extern crate lime_render as render;
extern crate lime_ui as ui;
extern crate lime_utils as utils;
extern crate shrev;
extern crate specs;
extern crate winit;

mod common;

use cassowary::strength::*;
use render::Color;
use shrev::EventChannel;
use specs::prelude::*;
use ui::draw::{Brush, Style};
use ui::layout::Position;
use ui::tree::{Node, Root};
use ui::widget::button::{Button, ButtonStyle};
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

    let btn_style = ButtonStyle {
        disabled: Brush::Color(Color::rgb(0.2, 0.2, 0.2)),
        normal: Brush::Color(Color::RED),
        focused: Brush::Color(Color::GREEN),
        pressed: Brush::Color(Color::BLUE),
    };

    let style = world.create_entity().with(btn_style).build();

    let rect = Node::with_parent(world.create_entity(), root)
        .with(pos)
        .with(cons)
        .with(Button::new(true))
        .build();

    Style::insert::<ButtonStyle>(rect, style, &mut world.write_storage()).unwrap();

    let mut quit = false;
    while !quit {
        events_loop.poll_events(|event| {
            match event {
                Event::WindowEvent {
                    event: WindowEvent::Closed,
                    ..
                } => quit = true,
                ev => world.write_resource::<EventChannel<_>>().single_write(ev),
            };
        });

        dispatcher.dispatch(&world.res);
    }
}
