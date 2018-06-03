extern crate cassowary;
extern crate env_logger;
extern crate lime_render as render;
extern crate lime_ui as ui;
extern crate lime_utils as utils;
extern crate specs;
extern crate winit;

mod common;

use std::iter;

use cassowary::strength::*;
use render::Color;
use specs::prelude::*;
use ui::layout::grid::Size;
use ui::layout::Grid;
use ui::{Brush, Node, Position, Root};
use winit::{Event, EventsLoop, WindowEvent};

fn create_rect(world: &mut World, parent: Entity, col: u32, row: u32, color: Color) -> Entity {
    let pos = Position::new();
    let mut cons = pos.constraints_builder()
        .min_size((100.0, 100.0), STRONG)
        .build();
    world
        .read_storage::<Grid>()
        .get(parent)
        .unwrap()
        .insert(col, row, &pos, &mut cons);

    Node::with_parent(world.create_entity(), parent)
        .with(pos)
        .with(cons)
        .with(Brush::Color(color))
        .build()
}

fn main() {
    env_logger::init();
    std::panic::set_hook(Box::new(utils::panic_hook));

    let mut events_loop = EventsLoop::new();
    let (mut world, mut dispatcher) = common::init(&events_loop);

    let root = world.read_resource::<Root>().entity();
    {
        let poss = world.read_storage();
        let (grid, cons) = Grid::new(
            poss.get(root).unwrap(),
            iter::repeat(Size::Auto).take(2),
            iter::repeat(Size::Auto).take(3),
        );
        world.write_storage().insert(root, grid).unwrap();
        world.write_storage().insert(root, cons).unwrap();
    }

    create_rect(&mut world, root, 0, 0, Color::RED);
    create_rect(&mut world, root, 1, 1, Color::GREEN);
    create_rect(&mut world, root, 0, 2, Color::BLUE);

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
