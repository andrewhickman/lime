mod keyboard;
mod mouse;

pub use self::keyboard::{KeyboardEvent, KeyboardFocus};
pub use self::mouse::{MouseEvent, MouseHover};

use render::d2::Point;
use shrev::{EventChannel, ReaderId};
use specs::prelude::*;
use winit::WindowEvent;

use {Node, Position, Root};

#[derive(Copy, Clone, Debug)]
pub struct Event {
    pub kind: EventKind,
    pub entity: Entity,
}

impl Event {
    fn keyboard(event: KeyboardEvent, entity: Entity) -> Self {
        Event {
            kind: EventKind::Keyboard(event),
            entity,
        }
    }

    fn mouse(event: MouseEvent, entity: Entity) -> Self {
        Event {
            kind: EventKind::Mouse(event),
            entity,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum EventKind {
    Keyboard(KeyboardEvent),
    Mouse(MouseEvent),
}

pub struct EventSystem {
    reader: ReaderId<WindowEvent>,
}

impl<'a> System<'a> for EventSystem {
    type SystemData = (
        ReadExpect<'a, Root>,
        Entities<'a>,
        ReadExpect<'a, KeyboardFocus>,
        WriteExpect<'a, MouseHover>,
        ReadExpect<'a, EventChannel<WindowEvent>>,
        WriteExpect<'a, EventChannel<Event>>,
        ReadStorage<'a, Node>,
        ReadStorage<'a, Position>,
    );

    fn run(
        &mut self,
        (root, ents, kb_focus, mut hover, win_events, mut events, nodes, poss): Self::SystemData,
    ) {
        let kb_entity = if ents.is_alive(kb_focus.entity) {
            kb_focus.entity
        } else {
            root.entity()
        };

        for win_event in win_events.read(&mut self.reader) {
            match *win_event {
                WindowEvent::KeyboardInput { input, .. } => {
                    if let Some(event) = KeyboardEvent::from_input(input) {
                        events.single_write(Event::keyboard(event, kb_entity));
                    }
                }
                WindowEvent::ReceivedCharacter(chr) => {
                    events.single_write(Event::keyboard(KeyboardEvent::Char(chr), kb_entity));
                }
                WindowEvent::CursorMoved {
                    position,
                    modifiers,
                    ..
                } => {
                    let point = Point(position.0 as f32, position.1 as f32);
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
                WindowEvent::CursorLeft { .. } => {
                    if let Some(ent) = hover.entity {
                        events.single_write(Event::mouse(MouseEvent::Exit, ent));
                    }
                    hover.entity = None;
                }
                WindowEvent::MouseInput {
                    state,
                    button,
                    modifiers,
                    ..
                } => {
                    if let Some(ent) = hover.entity {
                        events.single_write(Event::mouse(
                            MouseEvent::from_input(state, button, modifiers),
                            ent,
                        ));
                    }
                }
                _ => (),
            };
        }
    }
}
