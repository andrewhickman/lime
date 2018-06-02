mod keyboard;
mod mouse;
mod sys;

pub use self::keyboard::{KeyboardEvent, KeyboardFocus};
pub use self::mouse::{MouseEvent, MouseHover};
pub use self::sys::EventSystem;

use specs::prelude::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Event {
    pub kind: EventKind,
    pub entity: Entity,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum EventKind {
    Keyboard(KeyboardEvent),
    Mouse(MouseEvent),
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
