mod keyboard;
mod mouse;
mod sys;

pub use self::keyboard::{KeyboardEvent, KeyboardFocus};
pub use self::mouse::{MouseEvent, MouseHover};
pub use self::sys::EventSystem;

use specs::prelude::*;

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
