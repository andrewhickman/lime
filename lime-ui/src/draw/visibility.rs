use std::mem;

use shrev::EventChannel;
use specs::prelude::*;
use specs_mirror::{Mirrored, MirroredStorage};

#[derive(Copy, Clone, Debug)]
pub struct Visibility {
    state: VisibilityState,
}

impl Component for Visibility {
    type Storage = MirroredStorage<Self>;
}

impl Mirrored for Visibility {
    type Event = VisibilityEvent;
}

impl Visibility {
    pub fn new() -> Self {
        Visibility {
            state: VisibilityState::Visible,
        }
    }

    pub(crate) fn needs_draw(&self) -> bool {
        self.state == VisibilityState::Visible
    }

    pub fn get(&self) -> VisibilityState {
        self.state
    }

    pub fn set(
        &mut self,
        entity: Entity,
        new: VisibilityState,
        chan: &mut EventChannel<VisibilityEvent>,
    ) {
        let old = mem::replace(&mut self.state, new);
        if old != new {
            chan.single_write(VisibilityEvent { entity, old, new })
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum VisibilityState {
    Visible,
    Hidden,
    Collapsed,
}

pub struct VisibilityEvent {
    pub entity: Entity,
    pub old: VisibilityState,
    pub new: VisibilityState,
}

impl VisibilityEvent {
    pub(crate) fn needs_layout_changed(&self) -> Option<bool> {
        match (self.old, self.new) {
            (VisibilityState::Collapsed, VisibilityState::Collapsed) => None,
            (VisibilityState::Collapsed, _) => Some(true),
            (_, VisibilityState::Collapsed) => Some(false),
            (_, _) => None,
        }
    }
}
