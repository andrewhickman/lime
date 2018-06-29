mod style;
mod sys;

pub use self::style::{RadioButtonStyle, RadioButtonStyleSystem};
pub use self::sys::RadioButtonSystem;

use specs::prelude::*;

#[derive(Clone, Debug)]
pub struct RadioButton {
    group: Entity,
}

impl RadioButton {
    pub fn new(group: Entity) -> Self {
        RadioButton { group }
    }
}

#[derive(Clone, Debug)]
pub struct RadioButtonGroup {
    entities: Vec<Entity>,
}

impl RadioButtonGroup {
    pub fn new(entities: Vec<Entity>) -> Self {
        RadioButtonGroup { entities }
    }
}

impl Component for RadioButton {
    type Storage = HashMapStorage<Self>;
}

impl Component for RadioButtonGroup {
    type Storage = HashMapStorage<Self>;
}
