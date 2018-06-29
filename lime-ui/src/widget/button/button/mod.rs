mod style;
mod sys;

pub use self::style::{ButtonStyle, ButtonStyleSystem};
pub use self::sys::ButtonSystem;

use std::mem;

use shrev::EventChannel;
use specs::prelude::*;
use specs_mirror::{Mirrored, MirroredStorage};

#[derive(Clone, Debug, Deserialize)]
pub struct Button {
    state: ButtonState,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Deserialize)]
pub enum ButtonState {
    Normal,
    Focused,
    Pressed,
    Disabled,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ButtonEvent {
    pub entity: Entity,
    pub old: ButtonState,
    pub new: ButtonState,
}

impl Button {
    pub fn new(enabled: bool) -> Self {
        Button {
            state: if enabled {
                ButtonState::Normal
            } else {
                ButtonState::Disabled
            },
        }
    }

    pub fn state(&self) -> ButtonState {
        self.state
    }

    pub fn set_state(
        &mut self,
        entity: Entity,
        new: ButtonState,
        chan: &mut EventChannel<ButtonEvent>,
    ) {
        let old = mem::replace(&mut self.state, new);
        if old != new {
            chan.single_write(ButtonEvent { entity, old, new });
        }
    }
}

impl ButtonEvent {
    pub fn is_press(&self) -> bool {
        self.new == ButtonState::Pressed
    }
}

impl Component for Button {
    type Storage = MirroredStorage<Self, HashMapStorage<Self>>;
}

impl Mirrored for Button {
    type Event = ButtonEvent;
}
