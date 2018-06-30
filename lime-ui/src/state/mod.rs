#[cfg(test)]
mod tests;

use erased_serde as erased;
use serde::de as serde;
use shrev::EventChannel;
use specs::prelude::*;
use specs_mirror::{Mirrored, MirroredStorage};

use de::{DeserializeAndInsert, Seed};

/// State of a UI element.
#[derive(Copy, Clone, Default, Debug, Hash, PartialEq, PartialOrd, Eq, Ord)]
pub struct State {
    flags: StateFlags,
}

bitflags!{
    struct StateFlags: u8 {
        const NEEDS_LAYOUT = 0b0001;
        const NEEDS_DRAW   = 0b0010;
        const NEEDS_EVENTS = 0b0100;
    }
}

pub struct StateEvent {
    pub entity: Entity,
    flags: StateFlags,
    changed: StateFlags,
}

impl State {
    pub fn needs_events(&self) -> bool {
        self.flags.contains(StateFlags::NEEDS_EVENTS)
    }

    pub fn set_needs_events(
        &mut self,
        entity: Entity,
        value: bool,
        chan: &mut EventChannel<StateEvent>,
    ) {
        self.set(entity, StateFlags::NEEDS_EVENTS, value, chan)
    }

    pub fn needs_layout(&self) -> bool {
        self.flags.contains(StateFlags::NEEDS_EVENTS)
    }

    pub fn set_needs_layout(
        &mut self,
        entity: Entity,
        value: bool,
        chan: &mut EventChannel<StateEvent>,
    ) {
        self.set(entity, StateFlags::NEEDS_LAYOUT, value, chan)
    }

    pub fn needs_draw(&self) -> bool {
        self.flags.contains(StateFlags::NEEDS_DRAW)
    }

    pub fn set_needs_draw(
        &mut self,
        entity: Entity,
        value: bool,
        chan: &mut EventChannel<StateEvent>,
    ) {
        self.set(entity, StateFlags::NEEDS_DRAW, value, chan)
    }

    fn set(
        &mut self,
        entity: Entity,
        flag: StateFlags,
        value: bool,
        chan: &mut EventChannel<StateEvent>,
    ) {
        let old = self.flags;
        if value {
            self.flags.insert(flag.dependencies());
        } else {
            self.flags.remove(flag.dependants())
        }
        let new = self.flags;
        chan.single_write(StateEvent {
            entity,
            changed: old ^ new,
            flags: new,
        });
    }
}

impl Component for State {
    type Storage = MirroredStorage<Self, VecStorage<Self>>;
}

impl Mirrored for State {
    type Event = StateEvent;
}

impl DeserializeAndInsert for State {
    fn deserialize_and_insert<'de, 'a>(
        seed: Seed<'de, 'a>,
        deserializer: &mut erased::Deserializer<'de>,
    ) -> Result<(), erased::Error> {
        #[derive(Deserialize)]
        #[serde(rename = "State", rename_all = "snake_case")]
        enum StateDe {
            Collapsed = 0b0000,
            Hidden = 0b0001,
            Disabled = 0b0011,
            Enabled = 0b0111,
        }

        let state: StateDe = serde::Deserialize::deserialize(deserializer)?;
        let flags = StateFlags { bits: state as u8 };
        let res = WriteStorage::fetch(seed.res).insert(seed.entity, State { flags });
        if res.unwrap().is_some() {
            Err(serde::Error::custom(format!(
                "visibility defined twice for entity '{}'",
                seed.get_name(seed.entity)
            )))
        } else {
            Ok(())
        }
    }
}

impl Default for StateFlags {
    fn default() -> Self {
        StateFlags::all()
    }
}

impl StateFlags {
    fn dependants(self) -> Self {
        debug_assert_eq!(self.bits.count_ones(), 1);
        StateFlags::from_bits_truncate(!(self.bits - 1))
    }

    fn dependencies(self) -> Self {
        debug_assert_eq!(self.bits.count_ones(), 1);
        StateFlags {
            bits: (self.bits << 1) - 1,
        }
    }
}

impl StateEvent {
    pub fn needs_layout_changed(&self) -> Option<bool> {
        self.changed(StateFlags::NEEDS_LAYOUT)
    }

    pub fn needs_draw_changed(&self) -> Option<bool> {
        self.changed(StateFlags::NEEDS_DRAW)
    }

    pub fn needs_events_changed(&self) -> Option<bool> {
        self.changed(StateFlags::NEEDS_EVENTS)
    }

    fn changed(&self, flag: StateFlags) -> Option<bool> {
        if self.changed.contains(flag) {
            Some(self.flags.contains(flag))
        } else {
            None
        }
    }
}
