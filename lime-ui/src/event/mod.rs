#[cfg(test)]
pub mod tests;

mod keyboard;
mod mouse;
mod sys;

pub use self::keyboard::{KeyboardEvent, KeyboardFocus};
pub use self::mouse::{MouseEvent, MouseFocus};
pub use self::sys::EventSystem;

use specs::prelude::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Event {
    kind: EventKind,
    entity: Entity,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum EventKind {
    Keyboard(KeyboardEvent),
    Mouse(MouseEvent),
}

impl Event {
    pub fn entity(&self) -> Entity {
        self.entity
    }

    pub fn kind(&self) -> EventKind {
        self.kind
    }

    fn keyboard(entity: Entity, event: KeyboardEvent) -> Self {
        Event {
            entity,
            kind: EventKind::Keyboard(event),
        }
    }

    fn mouse(entity: Entity, event: MouseEvent) -> Self {
        Event {
            entity,
            kind: EventKind::Mouse(event),
        }
    }
}
