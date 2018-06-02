use render::d2::Point;
use specs::prelude::*;
use shrev::EventChannel;

use winit::{self, DeviceEvent, ElementState, WindowEvent, ModifiersState, MouseButton};

use event::{mouse, KeyboardFocus, KeyboardEvent, MouseEvent, MouseHover};
use {Event, Node, Position, Root};

pub struct EventSystem<'a>(pub &'a winit::Event);

impl<'a> System<'a> for EventSystem<'a> {
    type SystemData = (
        ReadExpect<'a, Root>,
        Entities<'a>,
        ReadExpect<'a, KeyboardFocus>,
        WriteExpect<'a, MouseHover>,
        WriteExpect<'a, EventChannel<Event>>,
        ReadStorage<'a, Node>,
        ReadStorage<'a, Position>,
    );

    fn run(
        &mut self,
        data: Self::SystemData,
    ) {
        match *self.0 {
            winit::Event::WindowEvent { ref event, .. } => match *event {
                WindowEvent::KeyboardInput { input, .. } => Self::keyboard_input(input, data),
                WindowEvent::ReceivedCharacter(chr) => Self::received_character(chr, data),
                WindowEvent::CursorMoved { position, modifiers, .. } => Self::cursor_moved(position, modifiers, data),
                WindowEvent::CursorLeft { .. } => Self::cursor_left(data),
                WindowEvent::MouseInput { state, button, modifiers, .. } => Self::mouse_input(state, button, modifiers, data),
                _ => (),
            },
            winit::Event::DeviceEvent { ref event, .. } => match *event {
                DeviceEvent::MouseMotion { delta, .. } => Self::mouse_motion(delta, data),
                _ => (),
            }
            _ => (),
        }
    }
}

impl<'a> EventSystem<'a> {
    fn keyboard_focus(root: ReadExpect<'a, Root>, ents: Entities<'a>, kb_focus: ReadExpect<'a, KeyboardFocus>) -> Entity {
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
        (root, ents, kb_focus, _, mut events, _, _): <Self as System<'a>>::SystemData
    ) {
        if let Some(event) = KeyboardEvent::from_input(input) {
            let ent = Self::keyboard_focus(root, ents, kb_focus);
            events.single_write(Event::keyboard(event, ent));
        }
    }

    fn received_character(
        chr: char,
        (root, ents, kb_focus, _, mut events, _, _): <Self as System<'a>>::SystemData
    ) {
        let ent = Self::keyboard_focus(root, ents, kb_focus);
        events.single_write(Event::keyboard(KeyboardEvent::Char(chr), ent));
    }

    fn cursor_moved(
        (x, y): (f64, f64),
        modifiers: ModifiersState,
        (root, _, _, mut hover, mut events, nodes, poss): <Self as System<'a>>::SystemData
    ) {
        let point = Point(x as f32, y as f32);
        let entity = mouse::hit_test(root.entity(), point, &nodes, &poss);

        if hover.entity != entity {
            if let Some(old) = hover.entity {
                events.single_write(Event::mouse(MouseEvent::Exit, old));
            }
            if let Some(new) = entity {
                events.single_write(Event::mouse(MouseEvent::Enter, new));
            }
            hover.entity = entity;
        }

        if let Some(ent) = hover.entity {
            events.single_write(Event::mouse(MouseEvent::Move(point, modifiers), ent));
        }
    }

    fn cursor_left(
        (_, _, _, mut hover, mut events, _, _): <Self as System<'a>>::SystemData
    ) {
        if let Some(ent) = hover.entity {
            events.single_write(Event::mouse(MouseEvent::Exit, ent));
        }
        hover.entity = None;
    }

    fn mouse_input(
        state: ElementState,
        button: MouseButton,
        modifiers: ModifiersState,
        (_, _, _, hover, mut events, _, _): <Self as System<'a>>::SystemData
    ) {
        if let Some(ent) = hover.entity {
            events.single_write(Event::mouse(
                MouseEvent::from_input(state, button, modifiers),
                ent,
            ));
        }
    }

    fn mouse_motion(
        (x, y): (f64, f64),
        (_, _, _, hover, mut events, _, _): <Self as System<'a>>::SystemData
    ) {
        if let Some(ent) = hover.entity {
            events.single_write(Event::mouse(
                MouseEvent::MoveRaw(x, y),
                ent,
            ));
        }
    }
}