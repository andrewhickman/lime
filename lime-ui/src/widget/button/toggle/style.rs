use shrev::ReaderId;
use specs::prelude::*;
use specs_mirror::{StorageExt, StorageMutExt};

use draw::{Brush, Style, StyleEvent};
use widget::button::{
    Button, ButtonEvent, ButtonState, ButtonStyle, ToggleButton, ToggleButtonEvent,
    ToggleButtonSystem,
};

#[derive(Component, Deserialize)]
#[storage(HashMapStorage)]
pub struct ToggleButtonStyle {
    pub on: ButtonStyle,
    pub off: ButtonStyle,
}

impl ToggleButtonStyle {
    pub fn brush(&self, state: (ButtonState, bool)) -> &Brush {
        match state.1 {
            false => self.on.brush(state.0),
            true => self.on.brush(state.0),
        }
    }
}

pub struct ToggleButtonStyleSystem {
    style_rx: ReaderId<StyleEvent>,
    btn_rx: ReaderId<ButtonEvent>,
    tgl_rx: ReaderId<ToggleButtonEvent>,
}

impl ToggleButtonStyleSystem {
    pub const NAME: &'static str = "ui::ToggleButtonStyle";

    pub(crate) fn add(world: &mut World, dispatcher: &mut DispatcherBuilder) {
        let style_rx = world.write_storage::<Style>().register_reader();
        let btn_rx = world.write_storage::<Button>().register_reader();
        let tgl_rx = world.write_storage::<ToggleButton>().register_reader();
        dispatcher.add(
            ToggleButtonStyleSystem {
                style_rx,
                btn_rx,
                tgl_rx,
            },
            ToggleButtonStyleSystem::NAME,
            &[ToggleButtonSystem::NAME],
        );
    }
}

impl<'a> System<'a> for ToggleButtonStyleSystem {
    type SystemData = (
        ReadStorage<'a, Button>,
        ReadStorage<'a, ToggleButton>,
        ReadStorage<'a, Style>,
        ReadStorage<'a, ToggleButtonStyle>,
        WriteStorage<'a, Brush>,
    );

    fn run(&mut self, (btns, tgls, styles, tgl_styles, mut brushes): Self::SystemData) {
        for event in btns.read_events(&mut self.btn_rx) {
            if let Some(style) = styles.get(event.entity) {
                if style.is::<ToggleButtonStyle>() {
                    if let (Some(tgl), Some(tgl_style)) =
                        (tgls.get(event.entity), tgl_styles.get(style.get()))
                    {
                        brushes
                            .insert(
                                event.entity,
                                tgl_style.brush((event.new, tgl.state())).clone(),
                            )
                            .ok();
                    }
                }
            }
        }

        for event in tgls.read_events(&mut self.tgl_rx) {
            if let Some(style) = styles.get(event.entity) {
                if style.is::<ToggleButtonStyle>() {
                    if let (Some(btn), Some(tgl_style)) =
                        (btns.get(event.entity), tgl_styles.get(style.get()))
                    {
                        brushes
                            .insert(
                                event.entity,
                                tgl_style.brush((btn.state(), event.state)).clone(),
                            )
                            .ok();
                    }
                }
            }
        }

        for event in styles.read_events(&mut self.style_rx) {
            if event.style.is::<ToggleButtonStyle>() {
                if let (Some(btn), Some(tgl), Some(btn_style)) = (
                    btns.get(event.entity),
                    tgls.get(event.entity),
                    tgl_styles.get(event.style.get()),
                ) {
                    brushes
                        .insert(
                            event.entity,
                            btn_style.brush((btn.state(), tgl.state())).clone(),
                        )
                        .ok();
                }
            }
        }
    }
}
