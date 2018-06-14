mod de;
mod style;
mod sys;
#[cfg(test)]
mod tests;

pub use self::style::{ButtonStyle, ButtonStyleSystem, ToggleButtonStyle, ToggleButtonStyleSystem};
pub use self::sys::ButtonSystem;

use std::sync::Arc;

use specs::error::Error;
use specs::prelude::*;
use specs_mirror::{Mirrored, MirroredStorage};

#[derive(Clone, Debug, Deserialize)]
pub struct Button {
    state: ButtonState,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ToggleButton {
    state: bool,
}

#[derive(Clone, Debug)]
pub struct RadioButton {
    group: Arc<[Entity]>,
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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ToggleButtonEvent {
    pub entity: Entity,
    pub state: bool,
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
}

impl ToggleButton {
    pub fn new(state: bool) -> Self {
        ToggleButton { state }
    }

    pub fn state(&self) -> bool {
        self.state
    }
}

impl RadioButton {
    pub fn create_group(
        storage: &mut WriteStorage<RadioButton>,
        entities: Arc<[Entity]>,
    ) -> Result<(), Error> {
        for &ent in entities.iter() {
            storage.insert(
                ent,
                RadioButton {
                    group: entities.clone(),
                },
            )?;
        }
        Ok(())
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

impl Component for ToggleButton {
    type Storage = MirroredStorage<Self, HashMapStorage<Self>>;
}

impl Mirrored for ToggleButton {
    type Event = ToggleButtonEvent;
}

impl Component for RadioButton {
    type Storage = HashMapStorage<Self>;
}
