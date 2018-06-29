use specs::prelude::*;
use specs_mirror::{StorageExt, StorageMutExt};
use winit::{ModifiersState, MouseButton};

use super::*;
use event::tests::emit_mouse_event;
use tests::init_test;
use tree::{Node, Root};

fn check_button_state(world: &mut World, button: Entity, state: ButtonState) {
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
    use event::MouseEvent::*;
    use widget::button::ButtonState::*;

    let (mut world, mut dispatcher) = init_test([0, 0].into());
    let mut reader = world.write_storage::<Button>().register_reader();
    let root = world.read_resource::<Root>().entity();

    let btn = Node::with_parent(world.create_entity(), root)
        .with(Button::new(true))
        .build();

    dispatcher.dispatch(&world.res);
    assert_eq!(
        world.read_storage::<Button>().get(btn).unwrap().state(),
        Normal
    );
    assert!(!was_clicked(&mut world, btn, &mut reader));

    emit_mouse_event(&mut world, btn, Enter);
    dispatcher.dispatch(&world.res);
    check_button_state(&mut world, btn, Focused);
    assert!(!was_clicked(&mut world, btn, &mut reader));

    emit_mouse_event(
        &mut world,
        btn,
        ButtonDown(MouseButton::Left, ModifiersState::default()),
    );
    dispatcher.dispatch(&world.res);
    check_button_state(&mut world, btn, Pressed);
    assert!(was_clicked(&mut world, btn, &mut reader));

    emit_mouse_event(
        &mut world,
        btn,
        ButtonUp(MouseButton::Left, ModifiersState::default()),
    );
    dispatcher.dispatch(&world.res);
    check_button_state(&mut world, btn, Focused);
    assert!(!was_clicked(&mut world, btn, &mut reader));

    emit_mouse_event(&mut world, btn, Exit);
    dispatcher.dispatch(&world.res);
    check_button_state(&mut world, btn, Normal);
    assert!(!was_clicked(&mut world, btn, &mut reader));

    emit_mouse_event(&mut world, btn, Enter);
    dispatcher.dispatch(&world.res);
    check_button_state(&mut world, btn, Focused);
    assert!(!was_clicked(&mut world, btn, &mut reader));

    emit_mouse_event(
        &mut world,
        btn,
        ButtonUp(MouseButton::Left, ModifiersState::default()),
    );
    dispatcher.dispatch(&world.res);
    check_button_state(&mut world, btn, Focused);
    assert!(!was_clicked(&mut world, btn, &mut reader));

    emit_mouse_event(
        &mut world,
        btn,
        ButtonDown(MouseButton::Left, ModifiersState::default()),
    );
    dispatcher.dispatch(&world.res);
    check_button_state(&mut world, btn, Pressed);
    assert!(was_clicked(&mut world, btn, &mut reader));

    emit_mouse_event(&mut world, btn, Exit);
    dispatcher.dispatch(&world.res);
    check_button_state(&mut world, btn, Normal);
    assert!(!was_clicked(&mut world, btn, &mut reader));
}

#[test]
fn toggle_button() {
    use event::MouseEvent::*;
    use widget::button::ButtonState::*;

    let (mut world, mut dispatcher) = init_test([1500, 1500].into());
    let mut btn_rdr = world.write_storage::<Button>().register_reader();
    let mut tgl_rdr = world.write_storage::<ToggleButton>().register_reader();
    let root = world.read_resource::<Root>().entity();

    let btn = Node::with_parent(world.create_entity(), root)
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

    emit_mouse_event(&mut world, btn, Enter);
    dispatcher.dispatch(&world.res);
    check_button_state(&mut world, btn, Focused);
    assert!(!was_clicked(&mut world, btn, &mut btn_rdr));
    assert_eq!(was_toggled(&mut world, btn, &mut tgl_rdr), None);

    emit_mouse_event(
        &mut world,
        btn,
        ButtonDown(MouseButton::Left, ModifiersState::default()),
    );
    dispatcher.dispatch(&world.res);
    check_button_state(&mut world, btn, Pressed);
    assert!(was_clicked(&mut world, btn, &mut btn_rdr));
    assert_eq!(was_toggled(&mut world, btn, &mut tgl_rdr), Some(true));

    emit_mouse_event(
        &mut world,
        btn,
        ButtonUp(MouseButton::Left, ModifiersState::default()),
    );
    dispatcher.dispatch(&world.res);
    check_button_state(&mut world, btn, Focused);
    assert!(!was_clicked(&mut world, btn, &mut btn_rdr));
    assert_eq!(was_toggled(&mut world, btn, &mut tgl_rdr), None);

    emit_mouse_event(&mut world, btn, Exit);
    dispatcher.dispatch(&world.res);
    check_button_state(&mut world, btn, Normal);
    assert!(!was_clicked(&mut world, btn, &mut btn_rdr));
    assert_eq!(was_toggled(&mut world, btn, &mut tgl_rdr), None);

    emit_mouse_event(&mut world, btn, Enter);
    dispatcher.dispatch(&world.res);
    check_button_state(&mut world, btn, Focused);
    assert!(!was_clicked(&mut world, btn, &mut btn_rdr));
    assert_eq!(was_toggled(&mut world, btn, &mut tgl_rdr), None);

    emit_mouse_event(
        &mut world,
        btn,
        ButtonUp(MouseButton::Left, ModifiersState::default()),
    );
    dispatcher.dispatch(&world.res);
    check_button_state(&mut world, btn, Focused);
    assert!(!was_clicked(&mut world, btn, &mut btn_rdr));
    assert_eq!(was_toggled(&mut world, btn, &mut tgl_rdr), None);

    emit_mouse_event(
        &mut world,
        btn,
        ButtonDown(MouseButton::Left, ModifiersState::default()),
    );
    dispatcher.dispatch(&world.res);
    check_button_state(&mut world, btn, Pressed);
    assert!(was_clicked(&mut world, btn, &mut btn_rdr));
    assert_eq!(was_toggled(&mut world, btn, &mut tgl_rdr), Some(false));

    emit_mouse_event(&mut world, btn, Exit);
    dispatcher.dispatch(&world.res);
    check_button_state(&mut world, btn, Normal);
    assert!(!was_clicked(&mut world, btn, &mut btn_rdr));
    assert_eq!(was_toggled(&mut world, btn, &mut tgl_rdr), None);
}

#[test]
fn radio_button() {
    use event::MouseEvent::*;
    use widget::button::ButtonState::*;

    let (mut world, mut dispatcher) = init_test([1500, 1500].into());
    let mut btn_rdr = world.write_storage::<Button>().register_reader();
    let mut tgl_rdr1 = world.write_storage::<ToggleButton>().register_reader();
    let mut tgl_rdr2 = world.write_storage::<ToggleButton>().register_reader();
    let mut tgl_rdr3 = world.write_storage::<ToggleButton>().register_reader();
    let root = world.read_resource::<Root>().entity();

    let group = world.create_entity().build();

    let btn1 = Node::with_parent(world.create_entity(), root)
        .with(Button::new(true))
        .with(ToggleButton::new(false))
        .with(RadioButton::new(group))
        .build();
    let btn2 = Node::with_parent(world.create_entity(), root)
        .with(Button::new(true))
        .with(ToggleButton::new(false))
        .with(RadioButton::new(group))
        .build();
    let btn3 = Node::with_parent(world.create_entity(), root)
        .with(Button::new(true))
        .with(ToggleButton::new(false))
        .with(RadioButton::new(group))
        .build();

    world
        .write_storage()
        .insert(group, RadioButtonGroup::new(vec![btn1, btn2, btn3]))
        .unwrap();

    emit_mouse_event(&mut world, btn1, Enter);
    dispatcher.dispatch(&world.res);
    check_button_state(&mut world, btn1, Focused);
    emit_mouse_event(
        &mut world,
        btn1,
        ButtonDown(MouseButton::Left, ModifiersState::default()),
    );
    dispatcher.dispatch(&world.res);
    check_button_state(&mut world, btn1, Pressed);
    assert!(was_clicked(&mut world, btn1, &mut btn_rdr));
    assert_eq!(was_toggled(&mut world, btn1, &mut tgl_rdr1), Some(true));
    assert_eq!(was_toggled(&mut world, btn2, &mut tgl_rdr2), None);
    assert_eq!(was_toggled(&mut world, btn3, &mut tgl_rdr3), None);
    emit_mouse_event(&mut world, btn1, Exit);
    dispatcher.dispatch(&world.res);
    check_button_state(&mut world, btn1, Normal);

    emit_mouse_event(&mut world, btn2, Enter);
    dispatcher.dispatch(&world.res);
    check_button_state(&mut world, btn2, Focused);
    emit_mouse_event(
        &mut world,
        btn2,
        ButtonDown(MouseButton::Left, ModifiersState::default()),
    );
    dispatcher.dispatch(&world.res);
    check_button_state(&mut world, btn2, Pressed);
    assert!(was_clicked(&mut world, btn2, &mut btn_rdr));
    assert_eq!(was_toggled(&mut world, btn1, &mut tgl_rdr1), Some(false));
    assert_eq!(was_toggled(&mut world, btn2, &mut tgl_rdr2), Some(true));
    assert_eq!(was_toggled(&mut world, btn3, &mut tgl_rdr3), None);
    emit_mouse_event(&mut world, btn2, Exit);
    dispatcher.dispatch(&world.res);
    check_button_state(&mut world, btn2, Normal);

    emit_mouse_event(&mut world, btn3, Enter);
    dispatcher.dispatch(&world.res);
    check_button_state(&mut world, btn3, Focused);
    emit_mouse_event(
        &mut world,
        btn3,
        ButtonDown(MouseButton::Left, ModifiersState::default()),
    );
    dispatcher.dispatch(&world.res);
    check_button_state(&mut world, btn3, Pressed);
    assert!(was_clicked(&mut world, btn3, &mut btn_rdr));
    assert_eq!(was_toggled(&mut world, btn1, &mut tgl_rdr1), None);
    assert_eq!(was_toggled(&mut world, btn2, &mut tgl_rdr2), Some(false));
    assert_eq!(was_toggled(&mut world, btn3, &mut tgl_rdr3), Some(true));

    emit_mouse_event(
        &mut world,
        btn3,
        ButtonUp(MouseButton::Left, ModifiersState::default()),
    );
    dispatcher.dispatch(&world.res);
    check_button_state(&mut world, btn3, Focused);
    emit_mouse_event(
        &mut world,
        btn3,
        ButtonDown(MouseButton::Left, ModifiersState::default()),
    );
    dispatcher.dispatch(&world.res);
    check_button_state(&mut world, btn3, Pressed);
    assert!(was_clicked(&mut world, btn3, &mut btn_rdr));
    assert_eq!(was_toggled(&mut world, btn1, &mut tgl_rdr1), None);
    assert_eq!(was_toggled(&mut world, btn2, &mut tgl_rdr2), None);
    assert_eq!(was_toggled(&mut world, btn3, &mut tgl_rdr3), None);
}
