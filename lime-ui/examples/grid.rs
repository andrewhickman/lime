extern crate cassowary;
extern crate env_logger;
extern crate lime_render as render;
extern crate lime_ui as ui;
extern crate lime_utils as utils;
extern crate shrev;
extern crate specs;
extern crate winit;

mod common;

use std::iter;

use cassowary::strength::*;
use render::Color;
use shrev::EventChannel;
use specs::prelude::*;
use ui::draw::Brush;
use ui::layout::{Constraints, Position};
use ui::tree::{Node, Root};
use ui::widget::grid::{Grid, Size};
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
    let mut events_loop = EventsLoop::new();
    let (mut world, mut dispatcher) = common::init(&events_loop);

    let root = world.read_resource::<Root>().entity();
    {
        let poss = world.read_storage();
        let pos = poss.get(root).unwrap();
        let mut cons = Constraints::new(pos);
        let grid = Grid::new(
            pos,
            &mut cons,
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
                    event: WindowEvent::CloseRequested,
                    ..
                } => quit = true,
                ev => world.write_resource::<EventChannel<_>>().single_write(ev),
            };
        });

        dispatcher.dispatch(&world.res);
    }
}
