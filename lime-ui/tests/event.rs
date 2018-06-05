extern crate cassowary;
extern crate lime_render as render;
extern crate lime_ui as ui;
extern crate lime_utils as utils;
extern crate shrev;
extern crate specs;
extern crate specs_mirror;
extern crate winit;

mod common;

use std::iter::FromIterator;
use std::mem;

use cassowary::strength::REQUIRED;
use cassowary::WeightedRelation::EQ;
use render::d2::Point;
use shrev::{EventChannel, ReaderId};
use specs::prelude::*;
use specs_mirror::{StorageExt, StorageMutExt};
use ui::event::{Button, ButtonEvent, ButtonState, MouseHover, ToggleButton, ToggleButtonEvent};
use ui::{Constraints, EventSystem, MouseEvent, Node, Position, Root};
use winit::{DeviceId, Event, ModifiersState, MouseButton, WindowEvent, WindowId};

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

fn find_mouse_event(events: &[ui::Event], entity: Entity, event: MouseEvent) -> bool {
    events
        .iter()
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

fn create_rect(world: &mut World, parent: Entity, l: i32, t: i32, w: i32, h: i32) -> EntityBuilder {
    let pos = Position::new();
    let cons = Constraints::from_iter(vec![
        pos.left() | EQ(REQUIRED) | l as f64,
        pos.top() | EQ(REQUIRED) | t as f64,
        pos.width() | EQ(REQUIRED) | w as f64,
        pos.height() | EQ(REQUIRED) | h as f64,
    ]);

    Node::with_parent(world.create_entity(), parent)
        .with(pos)
        .with(cons)
}

fn assert_target(
    world: &mut World,
    reader: &mut ReaderId<ui::Event>,
    old: Option<Entity>,
    new: Option<Entity>,
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

    let events: Vec<_> = {
        world
            .read_resource::<EventChannel<ui::Event>>()
            .read(reader)
            .cloned()
            .collect()
    };

    if let Some(ent) = old {
        if old != new {
            assert!(find_mouse_event(&events, ent, MouseEvent::Exit));
        }
    }

    if let Some(ent) = new {
        if old != new {
            assert!(find_mouse_event(&events, ent, MouseEvent::Enter));
        }
        assert!(find_mouse_event(
            &events,
            ent,
            MouseEvent::Move(Point(x as f32, y as f32), ModifiersState::default())
        ));
    } else {
        assert!(!mouse_moved(world, reader));
    }

    assert_eq!(world.read_resource::<MouseHover>().entity(), new);
}

#[test]
fn hover() {
    let (mut world, mut dispatcher) = init([1500, 1500].into());
    let mut rdr = world
        .write_resource::<EventChannel<ui::Event>>()
        .register_reader();
    let root = world.read_resource::<Root>().entity();

    let r1 = create_rect(&mut world, root, 0, 0, 1000, 1000).build();
    let r2 = create_rect(&mut world, root, 500, 500, 1000, 1000).build();

    dispatcher.dispatch(&world.res);

    assert_target(&mut world, &mut rdr, None, Some(r1), (250.0, 250.0));
    assert_target(&mut world, &mut rdr, Some(r1), Some(r1), (750.0, 250.0));
    assert_target(&mut world, &mut rdr, Some(r1), Some(root), (1250.0, 250.0));
    assert_target(&mut world, &mut rdr, Some(root), Some(r1), (250.0, 250.0));
    assert_target(&mut world, &mut rdr, Some(r1), Some(r2), (750.0, 750.0));
    assert_target(&mut world, &mut rdr, Some(r2), Some(r2), (1250.0, 1250.0));
    assert_target(&mut world, &mut rdr, Some(r2), Some(root), (250.0, 1250.0));
    assert_target(&mut world, &mut rdr, Some(root), Some(r2), (750.0, 1250.0));
    assert_target(&mut world, &mut rdr, Some(r2), Some(r2), (1250.0, 1250.0));

    assert_target(&mut world, &mut rdr, Some(r2), Some(r1), (0.0, 0.0));
    assert_target(&mut world, &mut rdr, Some(r1), None, (1500.0, 0.0));
    assert_target(&mut world, &mut rdr, None, None, (0.0, 1500.0));
    assert_target(&mut world, &mut rdr, None, None, (1500.0, 1500.0));
}

fn check_button_state(
    world: &mut World,
    dispatcher: &mut Dispatcher<'static, 'static>,
    button: Entity,
    event: MouseEvent,
    state: ButtonState,
) {
    world
        .write_resource::<EventChannel<ui::Event>>()
        .single_write(ui::Event {
            entity: button,
            kind: ui::EventKind::Mouse(event),
        });
    dispatcher.dispatch(&world.res);
    assert_eq!(
        world.read_storage::<Button>().get(button).unwrap().state(),
        state
    );
}

fn was_clicked(world: &mut World, button: Entity, reader: &mut ReaderId<ButtonEvent>) -> bool {
    let storage = world.read_storage::<Button>();
    for event in storage.read_events(reader) {
        if event.entity == button && event.is_press() {
            return true;
        }
    }
    false
}

fn was_toggled(
    world: &mut World,
    button: Entity,
    reader: &mut ReaderId<ToggleButtonEvent>,
) -> Option<bool> {
    let storage = world.read_storage::<ToggleButton>();
    for event in storage.read_events(reader) {
        if event.entity == button {
            assert_eq!(storage.get(button).unwrap().state(), event.state);
            return Some(event.state);
        }
    }
    None
}

#[test]
fn button() {
    use ButtonState::*;

    let (mut world, mut dispatcher) = init([1500, 1500].into());
    let mut reader = world.write_storage::<Button>().register_reader();
    let root = world.read_resource::<Root>().entity();

    let btn = create_rect(&mut world, root, 0, 0, 1000, 1000)
        .with(Button::new(true))
        .build();

    dispatcher.dispatch(&world.res);
    assert_eq!(
        world.read_storage::<Button>().get(btn).unwrap().state(),
        ButtonState::Normal
    );
    assert!(!was_clicked(&mut world, btn, &mut reader));

    check_button_state(&mut world, &mut dispatcher, btn, MouseEvent::Enter, Focused);
    assert!(!was_clicked(&mut world, btn, &mut reader));

    check_button_state(
        &mut world,
        &mut dispatcher,
        btn,
        MouseEvent::ButtonDown(MouseButton::Left, ModifiersState::default()),
        Pressed,
    );
    assert!(was_clicked(&mut world, btn, &mut reader));

    check_button_state(
        &mut world,
        &mut dispatcher,
        btn,
        MouseEvent::ButtonUp(MouseButton::Left, ModifiersState::default()),
        Focused,
    );
    assert!(!was_clicked(&mut world, btn, &mut reader));

    check_button_state(&mut world, &mut dispatcher, btn, MouseEvent::Exit, Normal);
    assert!(!was_clicked(&mut world, btn, &mut reader));

    check_button_state(&mut world, &mut dispatcher, btn, MouseEvent::Enter, Focused);
    assert!(!was_clicked(&mut world, btn, &mut reader));

    check_button_state(
        &mut world,
        &mut dispatcher,
        btn,
        MouseEvent::ButtonUp(MouseButton::Left, ModifiersState::default()),
        Focused,
    );
    assert!(!was_clicked(&mut world, btn, &mut reader));

    check_button_state(
        &mut world,
        &mut dispatcher,
        btn,
        MouseEvent::ButtonDown(MouseButton::Left, ModifiersState::default()),
        Pressed,
    );
    assert!(was_clicked(&mut world, btn, &mut reader));

    check_button_state(&mut world, &mut dispatcher, btn, MouseEvent::Exit, Normal);
    assert!(!was_clicked(&mut world, btn, &mut reader));
}

#[test]
fn toggle_button() {
    use ButtonState::*;

    let (mut world, mut dispatcher) = init([1500, 1500].into());
    let mut btn_rdr = world.write_storage::<Button>().register_reader();
    let mut tgl_rdr = world.write_storage::<ToggleButton>().register_reader();
    let root = world.read_resource::<Root>().entity();

    let btn = create_rect(&mut world, root, 0, 0, 1000, 1000)
        .with(Button::new(true))
        .with(ToggleButton::new(false))
        .build();

    dispatcher.dispatch(&world.res);
    assert_eq!(
        world.read_storage::<Button>().get(btn).unwrap().state(),
        ButtonState::Normal
    );
    assert!(!was_clicked(&mut world, btn, &mut btn_rdr));
    assert_eq!(was_toggled(&mut world, btn, &mut tgl_rdr), None);

    check_button_state(&mut world, &mut dispatcher, btn, MouseEvent::Enter, Focused);
    assert!(!was_clicked(&mut world, btn, &mut btn_rdr));
    assert_eq!(was_toggled(&mut world, btn, &mut tgl_rdr), None);

    check_button_state(
        &mut world,
        &mut dispatcher,
        btn,
        MouseEvent::ButtonDown(MouseButton::Left, ModifiersState::default()),
        Pressed,
    );
    assert!(was_clicked(&mut world, btn, &mut btn_rdr));
    assert_eq!(was_toggled(&mut world, btn, &mut tgl_rdr), Some(true));

    check_button_state(
        &mut world,
        &mut dispatcher,
        btn,
        MouseEvent::ButtonUp(MouseButton::Left, ModifiersState::default()),
        Focused,
    );
    assert!(!was_clicked(&mut world, btn, &mut btn_rdr));
    assert_eq!(was_toggled(&mut world, btn, &mut tgl_rdr), None);

    check_button_state(&mut world, &mut dispatcher, btn, MouseEvent::Exit, Normal);
    assert!(!was_clicked(&mut world, btn, &mut btn_rdr));
    assert_eq!(was_toggled(&mut world, btn, &mut tgl_rdr), None);

    check_button_state(&mut world, &mut dispatcher, btn, MouseEvent::Enter, Focused);
    assert!(!was_clicked(&mut world, btn, &mut btn_rdr));
    assert_eq!(was_toggled(&mut world, btn, &mut tgl_rdr), None);

    check_button_state(
        &mut world,
        &mut dispatcher,
        btn,
        MouseEvent::ButtonUp(MouseButton::Left, ModifiersState::default()),
        Focused,
    );
    assert!(!was_clicked(&mut world, btn, &mut btn_rdr));
    assert_eq!(was_toggled(&mut world, btn, &mut tgl_rdr), None);

    check_button_state(
        &mut world,
        &mut dispatcher,
        btn,
        MouseEvent::ButtonDown(MouseButton::Left, ModifiersState::default()),
        Pressed,
    );
    assert!(was_clicked(&mut world, btn, &mut btn_rdr));
    assert_eq!(was_toggled(&mut world, btn, &mut tgl_rdr), Some(false));

    check_button_state(&mut world, &mut dispatcher, btn, MouseEvent::Exit, Normal);
    assert!(!was_clicked(&mut world, btn, &mut btn_rdr));
    assert_eq!(was_toggled(&mut world, btn, &mut tgl_rdr), None);
}
