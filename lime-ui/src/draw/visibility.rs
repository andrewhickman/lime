use std::mem;

use shrev::EventChannel;
use specs::prelude::*;
use specs::world::Index;
use specs_mirror::{Mirrored, MirroredStorage};

#[derive(Copy, Clone, Debug)]
pub struct Visibility {
    state: VisibilityState,
}

impl Component for Visibility {
    type Storage = MirroredStorage<Self>;
}

impl Mirrored for Visibility {
    type State = VisibilityState;
    type Event = VisibilityEvent;

    fn insert(&mut self, _: &mut EventChannel<Self::Event>, _: Index) {}
    fn remove(&mut self, _: &mut EventChannel<Self::Event>, _: Index) {}

    fn modify(&mut self, chan: &mut EventChannel<Self::Event>, entity: Entity, new: Self::State) {
        let old = mem::replace(&mut self.state, new);
        if old != new {
            chan.single_write(VisibilityEvent { entity, old, new })
        }
    }
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
