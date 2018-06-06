use specs::prelude::*;
use winit::{ElementState, KeyboardInput, ModifiersState, VirtualKeyCode};

use tree::Root;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum KeyboardEvent {
    KeyUp(VirtualKeyCode, ModifiersState),
    KeyDown(VirtualKeyCode, ModifiersState),
    Char(char),
}

pub struct KeyboardFocus {
    pub(in event) entity: Entity,
}

impl KeyboardFocus {
    pub(crate) fn new(root: &Root) -> Self {
        KeyboardFocus {
            entity: root.entity(),
        }
    }

    pub fn set_entity(&mut self, entity: Entity) {
        self.entity = entity;
    }
}

impl KeyboardEvent {
    pub(in event) fn from_input(input: KeyboardInput) -> Option<Self> {
        if let Some(keycode) = input.virtual_keycode {
            Some(match input.state {
                ElementState::Pressed => KeyboardEvent::KeyDown(keycode, input.modifiers),
                ElementState::Released => KeyboardEvent::KeyUp(keycode, input.modifiers),
            })
        } else {
            None
        }
    }
}
