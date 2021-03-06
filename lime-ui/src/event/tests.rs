use std::mem;

use cassowary::strength::REQUIRED;
use cassowary::WeightedRelation::EQ;
use render::d2::Point;
use shrev::EventChannel;
use winit::dpi::LogicalPosition;
use winit::{self, ModifiersState, WindowEvent};

use super::*;
use layout::Position;
use tests::init_test;
use tree::{Node, Root};

pub fn emit_mouse_event(world: &mut World, entity: Entity, event: MouseEvent) {
    world
        .write_resource::<EventChannel<Event>>()
        .single_write(Event::mouse(entity, event));
}

pub fn emit_keyboard_event(world: &mut World, entity: Entity, event: KeyboardEvent) {
    world
        .write_resource::<EventChannel<Event>>()
        .single_write(Event::keyboard(entity, event));
}

pub fn run_window_event(world: &mut World, dispatcher: &mut Dispatcher, event: WindowEvent) {
    world
        .write_resource::<EventChannel<winit::Event>>()
        .single_write(winit::Event::WindowEvent {
            event,
            window_id: unsafe { mem::zeroed() },
        });
    dispatcher.dispatch(&world.res);
}

fn create_rect(
    world: &mut World,
    parent: Entity,
    left: f64,
    top: f64,
    width: f64,
    height: f64,
) -> EntityBuilder {
    let pos = Position::new();
    let cons = pos.constraints_builder()
        .with(pos.left_var() | EQ(REQUIRED) | left)
        .with(pos.top_var() | EQ(REQUIRED) | top)
        .with(pos.width_var() | EQ(REQUIRED) | width)
        .with(pos.height_var() | EQ(REQUIRED) | height)
        .build();

    Node::with_parent(world.create_entity(), parent)
        .with(pos)
        .with(cons)
}

fn find_mouse_event(events: &[Event], entity: Entity, event: MouseEvent) -> bool {
    events
        .iter()
        .find(|ev| ev.entity() == entity && ev.kind() == EventKind::Mouse(event))
        .is_some()
}

fn assert_mouse_focus(
    world: &mut World,
    dispatcher: &mut Dispatcher,
    reader: &mut ReaderId<Event>,
    old: Option<Entity>,
    new: Option<Entity>,
    (x, y): (f64, f64),
) {
    run_window_event(
        world,
        dispatcher,
        WindowEvent::CursorMoved {
            position: LogicalPosition::new(x, y),
            modifiers: ModifiersState::default(),
            device_id: unsafe { mem::zeroed() },
        },
    );

    let events: Vec<_> = {
        world
            .read_resource::<EventChannel<Event>>()
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

    assert_eq!(world.read_resource::<MouseFocus>().entity(), new);
}

fn mouse_moved(world: &mut World, reader: &mut ReaderId<Event>) -> bool {
    world
        .read_resource::<EventChannel<Event>>()
        .read(reader)
        .find(|&ev| match ev.kind() {
            EventKind::Mouse(MouseEvent::Move(_, _)) => true,
            _ => false,
        })
        .is_some()
}

#[test]
fn mouse_focus() {
    let (mut world, mut dispatcher) = init_test();
    let mut rdr = world
        .write_resource::<EventChannel<Event>>()
        .register_reader();
    let root = world.read_resource::<Root>().entity();

    let r1 = create_rect(&mut world, root, 0.0, 0.0, 1000.0, 1000.0).build();
    let r2 = create_rect(&mut world, root, 500.0, 500.0, 1000.0, 1000.0).build();

    run_window_event(
        &mut world,
        &mut dispatcher,
        WindowEvent::Resized((1500, 1500).into()),
    );

    assert_mouse_focus(
        &mut world,
        &mut dispatcher,
        &mut rdr,
        None,
        Some(r1),
        (250.0, 250.0),
    );
    assert_mouse_focus(
        &mut world,
        &mut dispatcher,
        &mut rdr,
        Some(r1),
        Some(r1),
        (750.0, 250.0),
    );
    assert_mouse_focus(
        &mut world,
        &mut dispatcher,
        &mut rdr,
        Some(r1),
        Some(root),
        (1250.0, 250.0),
    );
    assert_mouse_focus(
        &mut world,
        &mut dispatcher,
        &mut rdr,
        Some(root),
        Some(r1),
        (250.0, 250.0),
    );
    assert_mouse_focus(
        &mut world,
        &mut dispatcher,
        &mut rdr,
        Some(r1),
        Some(r2),
        (750.0, 750.0),
    );
    assert_mouse_focus(
        &mut world,
        &mut dispatcher,
        &mut rdr,
        Some(r2),
        Some(r2),
        (1250.0, 1250.0),
    );
    assert_mouse_focus(
        &mut world,
        &mut dispatcher,
        &mut rdr,
        Some(r2),
        Some(root),
        (250.0, 1250.0),
    );
    assert_mouse_focus(
        &mut world,
        &mut dispatcher,
        &mut rdr,
        Some(root),
        Some(r2),
        (750.0, 1250.0),
    );
    assert_mouse_focus(
        &mut world,
        &mut dispatcher,
        &mut rdr,
        Some(r2),
        Some(r2),
        (1250.0, 1250.0),
    );

    assert_mouse_focus(
        &mut world,
        &mut dispatcher,
        &mut rdr,
        Some(r2),
        Some(r1),
        (0.0, 0.0),
    );
    assert_mouse_focus(
        &mut world,
        &mut dispatcher,
        &mut rdr,
        Some(r1),
        None,
        (1500.0, 0.0),
    );
    assert_mouse_focus(
        &mut world,
        &mut dispatcher,
        &mut rdr,
        None,
        None,
        (0.0, 1500.0),
    );
    assert_mouse_focus(
        &mut world,
        &mut dispatcher,
        &mut rdr,
        None,
        None,
        (1500.0, 1500.0),
    );
}
