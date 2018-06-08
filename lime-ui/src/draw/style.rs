use std::sync::Arc;

use specs::prelude::*;
use specs_mirror::{StorageExt, StorageMutExt};

use draw::Brush;
use widget::button::{Button, ButtonEvent, ButtonState};

pub struct Style {
    def: Arc<StyleDef>,
}

impl Style {
    pub fn new(def: Arc<StyleDef>) -> Self {
        Style { def }
    }
}

impl Component for Style {
    type Storage = DenseVecStorage<Self>;
}

pub struct StyleDef {
    pub btn_normal: Brush,
    pub btn_focused: Brush,
    pub btn_pressed: Brush,
    pub btn_disabled: Brush,
}

impl StyleDef {
    pub fn button(&self, state: ButtonState) -> &Brush {
        match state {
            ButtonState::Normal => &self.btn_normal,
            ButtonState::Focused => &self.btn_focused,
            ButtonState::Pressed => &self.btn_pressed,
            ButtonState::Disabled => &self.btn_disabled,
        }
    }
}

pub struct StyleSystem {
    btn_rx: ReaderId<ButtonEvent>,
}

impl StyleSystem {
    pub const NAME: &'static str = "ui::style";

    pub(crate) fn new(world: &mut World) -> Self {
        let btn_rx = world.write_storage::<Button>().register_reader();
        StyleSystem { btn_rx }
    }
}

impl<'a> System<'a> for StyleSystem {
    type SystemData = (
        WriteStorage<'a, Brush>,
        ReadStorage<'a, Style>,
        ReadStorage<'a, Button>,
    );

    fn run(&mut self, (mut brushes, styles, btns): Self::SystemData) {
        for event in btns.read_events(&mut self.btn_rx) {
            if let (Some(brush), Some(style)) =
                (brushes.get_mut(event.entity), styles.get(event.entity))
            {
                brush.clone_from(style.def.button(event.new));
            }
        }
    }
}
