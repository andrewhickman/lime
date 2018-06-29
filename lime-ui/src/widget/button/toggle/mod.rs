mod style;
mod sys;

pub use self::style::{ToggleButtonStyle, ToggleButtonStyleSystem};
pub use self::sys::ToggleButtonSystem;

use shrev::EventChannel;
use specs::prelude::*;
use specs_mirror::{Mirrored, MirroredStorage};

#[derive(Clone, Debug, Deserialize)]
pub struct ToggleButton {
    state: bool,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ToggleButtonEvent {
    pub entity: Entity,
    pub state: bool,
}

impl ToggleButton {
    pub fn new(state: bool) -> Self {
        ToggleButton { state }
    }

    pub fn state(&self) -> bool {
        self.state
    }

    pub fn set_state(
        &mut self,
        entity: Entity,
        state: bool,
        chan: &mut EventChannel<ToggleButtonEvent>,
    ) {
        if self.state != state {
            self.state = state;
            chan.single_write(ToggleButtonEvent { entity, state });
        }
    }
}

impl Component for ToggleButton {
    type Storage = MirroredStorage<Self, HashMapStorage<Self>>;
}

impl Mirrored for ToggleButton {
    type Event = ToggleButtonEvent;
}
