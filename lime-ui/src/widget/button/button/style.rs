use shrev::ReaderId;
use specs::prelude::*;
use specs_mirror::{StorageExt, StorageMutExt};

use draw::{Brush, Style, StyleEvent};
use widget::button::{Button, ButtonEvent, ButtonState, ButtonSystem};

#[derive(Component, Deserialize)]
#[storage(HashMapStorage)]
pub struct ButtonStyle {
    pub disabled: Brush,
    pub normal: Brush,
    pub focused: Brush,
    pub pressed: Brush,
}

impl ButtonStyle {
    pub fn brush(&self, state: ButtonState) -> &Brush {
        match state {
            ButtonState::Disabled => &self.disabled,
            ButtonState::Normal => &self.normal,
            ButtonState::Focused => &self.focused,
            ButtonState::Pressed => &self.pressed,
        }
    }
}

pub struct ButtonStyleSystem {
    style_rx: ReaderId<StyleEvent>,
    btn_rx: ReaderId<ButtonEvent>,
}

impl ButtonStyleSystem {
    pub const NAME: &'static str = "ui::ButtonStyle";

    pub(crate) fn add(world: &mut World, dispatcher: &mut DispatcherBuilder) {
        let style_rx = world.write_storage::<Style>().register_reader();
        let btn_rx = world.write_storage::<Button>().register_reader();
        dispatcher.add(
            ButtonStyleSystem { style_rx, btn_rx },
            ButtonStyleSystem::NAME,
            &[ButtonSystem::NAME],
        );
    }
}

impl<'a> System<'a> for ButtonStyleSystem {
    type SystemData = (
        ReadStorage<'a, Button>,
        ReadStorage<'a, Style>,
        ReadStorage<'a, ButtonStyle>,
        WriteStorage<'a, Brush>,
    );

    fn run(&mut self, (btns, styles, btn_styles, mut brushes): Self::SystemData) {
        for event in btns.read_events(&mut self.btn_rx) {
            if let Some(style) = styles.get(event.entity) {
                if style.is::<ButtonStyle>() {
                    if let Some(btn_style) = btn_styles.get(style.get()) {
                        brushes
                            .insert(event.entity, btn_style.brush(event.new).clone())
                            .ok();
                    }
                }
            }
        }

        for event in styles.read_events(&mut self.style_rx) {
            if event.style.is::<ButtonStyle>() {
                if let (Some(btn), Some(btn_style)) =
                    (btns.get(event.entity), btn_styles.get(event.style.get()))
                {
                    brushes
                        .insert(event.entity, btn_style.brush(btn.state()).clone())
                        .ok();
                }
            }
        }
    }
}
