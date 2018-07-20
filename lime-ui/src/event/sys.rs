use render::d2::Point;
use shrev::{EventChannel, ReaderId};
use specs::prelude::*;
use specs_mirror::{StorageExt, StorageMutExt};

use winit::{self, DeviceEvent, ElementState, ModifiersState, MouseButton, WindowEvent};

use event::{mouse, Event, KeyboardEvent, KeyboardFocus, MouseEvent, MouseFocus};
use layout::Position;
use tree::{Node, Root};
use {State, StateEvent};

pub struct EventSystem {
    event_rx: ReaderId<winit::Event>,
    state_rx: ReaderId<StateEvent>,
}

impl EventSystem {
    pub const NAME: &'static str = "ui::Event";

    pub(crate) fn add(world: &mut World, dispatcher: &mut DispatcherBuilder) {
        let event_rx = world
            .write_resource::<EventChannel<winit::Event>>()
            .register_reader();
        let state_rx = world.write_storage::<State>().register_reader();
        dispatcher.add(EventSystem { event_rx, state_rx }, EventSystem::NAME, &[]);
    }
}

fn keyboard_focus(root: &Root, ents: &Entities, kb_focus: &KeyboardFocus) -> Entity {
    if ents.is_alive(kb_focus.entity) {
        kb_focus.entity
    } else {
        let root_ent = root.entity();
        debug_assert!(ents.is_alive(root_ent));
        root_ent
    }
}

fn keyboard_input(
    input: winit::KeyboardInput,
    root: &Root,
    ents: &Entities,
    kb_focus: &KeyboardFocus,
    events: &mut EventChannel<Event>,
    states: &ReadStorage<State>,
) {
    if let Some(event) = KeyboardEvent::from_input(input) {
        let ent = keyboard_focus(root, ents, kb_focus);
        if states.get(ent).map(State::needs_events).unwrap_or(true) {
            events.single_write(Event::keyboard(ent, event));
        }
    }
}

fn received_character(
    chr: char,
    root: &Root,
    ents: &Entities,
    kb_focus: &KeyboardFocus,
    events: &mut EventChannel<Event>,
    states: &ReadStorage<State>,
) {
    let ent = keyboard_focus(root, ents, kb_focus);
    if states.get(ent).map(State::needs_events).unwrap_or(true) {
        events.single_write(Event::keyboard(ent, KeyboardEvent::Char(chr)));
    }
}

fn cursor_update(
    root: &Root,
    hover: &mut MouseFocus,
    events: &mut EventChannel<Event>,
    nodes: &ReadStorage<Node>,
    poss: &ReadStorage<Position>,
    states: &ReadStorage<State>,
) {
    let entity = mouse::hit_test(root.entity(), hover.point, &nodes, &poss, &states);

    if hover.entity != entity {
        if let Some(old) = hover.entity {
            if states.get(old).map(State::needs_events).unwrap_or(true) {
                events.single_write(Event::mouse(old, MouseEvent::Exit));
            }
        }
        if let Some(new) = entity {
            if states.get(new).map(State::needs_events).unwrap_or(true) {
                events.single_write(Event::mouse(new, MouseEvent::Enter));
            }
        }
        hover.entity = entity;
    }
}

fn cursor_moved(
    point: Point,
    modifiers: ModifiersState,
    root: &Root,
    hover: &mut MouseFocus,
    events: &mut EventChannel<Event>,
    nodes: &ReadStorage<Node>,
    poss: &ReadStorage<Position>,
    states: &ReadStorage<State>,
) {
    hover.point = point;
    cursor_update(root, hover, events, nodes, poss, states);
    if let Some(ent) = hover.entity {
        if states.get(ent).map(State::needs_events).unwrap_or(true) {
            events.single_write(Event::mouse(ent, MouseEvent::Move(hover.point, modifiers)));
        }
    }
}

fn cursor_left(
    hover: &mut MouseFocus,
    events: &mut EventChannel<Event>,
    states: &ReadStorage<State>,
) {
    if let Some(ent) = hover.entity {
        if states.get(ent).map(State::needs_events).unwrap_or(true) {
            events.single_write(Event::mouse(ent, MouseEvent::Exit));
        }
    }
    hover.entity = None;
}

fn mouse_input(
    state: ElementState,
    button: MouseButton,
    modifiers: ModifiersState,
    hover: &mut MouseFocus,
    events: &mut EventChannel<Event>,
) {
    if let Some(ent) = hover.entity {
        events.single_write(Event::mouse(
            ent,
            MouseEvent::from_input(state, button, modifiers),
        ));
    }
}

fn mouse_motion((x, y): (f64, f64), hover: &mut MouseFocus, events: &mut EventChannel<Event>) {
    if let Some(ent) = hover.entity {
        events.single_write(Event::mouse(ent, MouseEvent::MoveRaw(x, y)));
    }
}

impl<'a> System<'a> for EventSystem {
    type SystemData = (
        ReadExpect<'a, EventChannel<winit::Event>>,
        ReadExpect<'a, Root>,
        Entities<'a>,
        ReadExpect<'a, KeyboardFocus>,
        WriteExpect<'a, MouseFocus>,
        WriteExpect<'a, EventChannel<Event>>,
        ReadStorage<'a, Node>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, State>,
    );

    fn run(
        &mut self,
        (win_events, root, ents, kb_focus, mut hover, mut events, nodes, poss, states): Self::SystemData,
){
        if states
            .read_events(&mut self.state_rx)
            .any(|ev| ev.needs_draw_changed().is_some())
        {
            cursor_update(&root, &mut hover, &mut events, &nodes, &poss, &states)
        }

        for win_event in win_events.read(&mut self.event_rx) {
            match win_event {
                winit::Event::WindowEvent { ref event, .. } => match *event {
                    WindowEvent::KeyboardInput { input, .. } => {
                        keyboard_input(input, &root, &ents, &kb_focus, &mut events, &states)
                    }
                    WindowEvent::ReceivedCharacter(chr) => {
                        received_character(chr, &root, &ents, &kb_focus, &mut events, &states)
                    }
                    WindowEvent::CursorMoved {
                        position,
                        modifiers,
                        ..
                    } => {
                        cursor_moved(
                            position.into(),
                            modifiers,
                            &root,
                            &mut hover,
                            &mut events,
                            &nodes,
                            &poss,
                            &states,
                        );
                    }
                    WindowEvent::CursorLeft { .. } => cursor_left(&mut hover, &mut events, &states),
                    WindowEvent::MouseInput {
                        state,
                        button,
                        modifiers,
                        ..
                    } => mouse_input(state, button, modifiers, &mut hover, &mut events),
                    _ => (),
                },
                winit::Event::DeviceEvent { ref event, .. } => match *event {
                    DeviceEvent::MouseMotion { delta, .. } => {
                        mouse_motion(delta, &mut hover, &mut events)
                    }
                    _ => (),
                },
                _ => (),
            }
        }
    }
}
