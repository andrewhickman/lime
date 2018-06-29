use shrev::ReaderId;
use specs::prelude::*;
use specs_mirror::{StorageExt, StorageMutExt};

use draw::{Brush, Style, StyleEvent};
use widget::button::{
    Button, ButtonEvent, ButtonState, RadioButtonSystem, ToggleButton, ToggleButtonEvent,
    ToggleButtonStyle,
};

#[derive(Component, Deserialize)]
pub struct RadioButtonStyle {
    #[serde(flatten)]
    pub style: ToggleButtonStyle,
}

impl RadioButtonStyle {
    pub fn brush(&self, state: (ButtonState, bool)) -> &Brush {
        self.style.brush(state)
    }
}

pub struct RadioButtonStyleSystem {
    style_rx: ReaderId<StyleEvent>,
    btn_rx: ReaderId<ButtonEvent>,
    tgl_rx: ReaderId<ToggleButtonEvent>,
}

impl RadioButtonStyleSystem {
    pub const NAME: &'static str = "ui::RadioButtonStyle";

    pub(crate) fn add(world: &mut World, dispatcher: &mut DispatcherBuilder) {
        let style_rx = world.write_storage::<Style>().register_reader();
        let btn_rx = world.write_storage::<Button>().register_reader();
        let tgl_rx = world.write_storage::<ToggleButton>().register_reader();
        dispatcher.add(
            RadioButtonStyleSystem {
                style_rx,
                btn_rx,
                tgl_rx,
            },
            RadioButtonStyleSystem::NAME,
            &[RadioButtonSystem::NAME],
        );
    }
}

impl<'a> System<'a> for RadioButtonStyleSystem {
    type SystemData = (
        ReadStorage<'a, Button>,
        ReadStorage<'a, ToggleButton>,
        ReadStorage<'a, Style>,
        ReadStorage<'a, RadioButtonStyle>,
        WriteStorage<'a, Brush>,
    );

    fn run(&mut self, (btns, tgls, styles, rad_styles, mut brushes): Self::SystemData) {
        for event in btns.read_events(&mut self.btn_rx) {
            if let Some(style) = styles.get(event.entity) {
                if style.is::<RadioButtonStyle>() {
                    if let (Some(tgl), Some(rad_style)) =
                        (tgls.get(event.entity), rad_styles.get(style.get()))
                    {
                        brushes
                            .insert(
                                event.entity,
                                rad_style.brush((event.new, tgl.state())).clone(),
                            )
                            .ok();
                    }
                }
            }
        }

        for event in tgls.read_events(&mut self.tgl_rx) {
            if let Some(style) = styles.get(event.entity) {
                if style.is::<RadioButtonStyle>() {
                    if let (Some(btn), Some(rad_style)) =
                        (btns.get(event.entity), rad_styles.get(style.get()))
                    {
                        brushes
                            .insert(
                                event.entity,
                                rad_style.brush((btn.state(), event.state)).clone(),
                            )
                            .ok();
                    }
                }
            }
        }

        for event in styles.read_events(&mut self.style_rx) {
            if event.style.is::<RadioButtonStyle>() {
                if let (Some(btn), Some(tgl), Some(rad_style)) = (
                    btns.get(event.entity),
                    tgls.get(event.entity),
                    rad_styles.get(event.style.get()),
                ) {
                    brushes
                        .insert(
                            event.entity,
                            rad_style.brush((btn.state(), tgl.state())).clone(),
                        )
                        .ok();
                }
            }
        }
    }
}
