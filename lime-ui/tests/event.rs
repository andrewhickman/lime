extern crate cassowary;
extern crate lime_render as render;
extern crate lime_ui as ui;
extern crate lime_utils as utils;
extern crate shrev;
extern crate specs;
extern crate winit;

mod common;

use std::mem;

use cassowary::strength::REQUIRED;
use cassowary::WeightedRelation::EQ;
use render::d2::Point;
use shrev::{EventChannel, ReaderId};
use specs::prelude::*;
use ui::event::MouseHover;
use ui::{Constraints, EventSystem, MouseEvent, Node, Position, Root};
use winit::{DeviceId, Event, ModifiersState, WindowEvent, WindowId};

use common::init;

fn window_id() -> WindowId {
    unsafe { mem::zeroed() }
}

fn device_id() -> DeviceId {
    unsafe { mem::zeroed() }
}

fn push_event(world: &mut World, event: WindowEvent) {
    EventSystem(&Event::WindowEvent {
        event,
        window_id: window_id(),
    }).run_now(&mut world.res)
}

fn find_mouse_event(
    world: &mut World,
    reader: &mut ReaderId<ui::Event>,
    entity: Entity,
    event: MouseEvent,
) -> bool {
    world
        .read_resource::<EventChannel<ui::Event>>()
        .read(reader)
        .find(|&ev| {
            *ev == ui::Event {
                kind: ui::EventKind::Mouse(event),
                entity,
            }
        })
        .is_some()
}

fn mouse_moved(world: &mut World, reader: &mut ReaderId<ui::Event>) -> bool {
    world
        .read_resource::<EventChannel<ui::Event>>()
        .read(reader)
        .find(|&ev| match ev.kind {
            ui::EventKind::Mouse(e @ MouseEvent::Move(_, _)) => {
                println!("ev: {:?}, ent: {:?}", e, ev.entity);
                true
            }
            _ => false,
        })
        .is_some()
}

fn create_rect(world: &mut World, parent: Entity, l: i32, t: i32, w: i32, h: i32) -> Entity {
    let pos = Position::new();
    let cons = Constraints::new(vec![
        pos.left() | EQ(REQUIRED) | l as f64,
        pos.top() | EQ(REQUIRED) | t as f64,
        pos.width() | EQ(REQUIRED) | w as f64,
        pos.height() | EQ(REQUIRED) | h as f64,
    ]);

    Node::with_parent(world.create_entity(), parent)
        .with(pos)
        .with(cons)
        .build()
}

fn assert_target(
    world: &mut World,
    reader: &mut ReaderId<ui::Event>,
    target: Option<Entity>,
    (x, y): (f64, f64),
) {
    push_event(
        world,
        WindowEvent::CursorMoved {
            position: (x, y),
            modifiers: ModifiersState::default(),
            device_id: device_id(),
        },
    );

    if let Some(ent) = target {
        assert!(find_mouse_event(
            world,
            reader,
            ent,
            MouseEvent::Move(Point(x as f32, y as f32), ModifiersState::default())
        ));
    } else {
        assert!(!mouse_moved(world, reader));
    }

    assert_eq!(world.read_resource::<MouseHover>().entity(), target);
}

#[test]
fn hover() {
    let (mut world, mut dispatcher) = init([1500, 1500].into());
    let mut reader = world
        .write_resource::<EventChannel<ui::Event>>()
        .register_reader();
    let root = world.read_resource::<Root>().entity();

    let r1 = create_rect(&mut world, root, 0, 0, 1000, 1000);
    let r2 = create_rect(&mut world, root, 500, 500, 1000, 1000);

    dispatcher.dispatch(&mut world.res);

    assert_target(&mut world, &mut reader, Some(r1), (250.0, 250.0));
    assert_target(&mut world, &mut reader, Some(r1), (750.0, 250.0));
    assert_target(&mut world, &mut reader, Some(root), (1250.0, 250.0));
    assert_target(&mut world, &mut reader, Some(r1), (250.0, 250.0));
    assert_target(&mut world, &mut reader, Some(r2), (750.0, 750.0));
    assert_target(&mut world, &mut reader, Some(r2), (1250.0, 1250.0));
    assert_target(&mut world, &mut reader, Some(root), (250.0, 1250.0));
    assert_target(&mut world, &mut reader, Some(r2), (750.0, 1250.0));
    assert_target(&mut world, &mut reader, Some(r2), (1250.0, 1250.0));

    assert_target(&mut world, &mut reader, Some(r1), (0.0, 0.0));
    assert_target(&mut world, &mut reader, None, (1500.0, 0.0));
    assert_target(&mut world, &mut reader, None, (0.0, 1500.0));
    assert_target(&mut world, &mut reader, None, (1500.0, 1500.0));
}
