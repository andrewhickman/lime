use shrev::ReaderId;
use specs::prelude::*;
use specs_mirror::{StorageExt, StorageMutExt};

use draw::{Brush, Style, StyleEvent};
use widget::button::{
    Button, ButtonEvent, ButtonState, ButtonSystem, ToggleButton, ToggleButtonEvent,
};

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

#[derive(Component, Deserialize)]
#[storage(HashMapStorage)]
pub struct ToggleButtonStyle {
    pub disabled_off: Brush,
    pub normal_off: Brush,
    pub focused_off: Brush,
    pub pressed_off: Brush,
    pub disabled_on: Brush,
    pub normal_on: Brush,
    pub focused_on: Brush,
    pub pressed_on: Brush,
}

impl ToggleButtonStyle {
    pub fn brush(&self, state: (ButtonState, bool)) -> &Brush {
        match state {
            (ButtonState::Disabled, false) => &self.disabled_off,
            (ButtonState::Normal, false) => &self.normal_off,
            (ButtonState::Focused, false) => &self.focused_off,
            (ButtonState::Pressed, false) => &self.pressed_off,
            (ButtonState::Disabled, true) => &self.disabled_on,
            (ButtonState::Normal, true) => &self.normal_on,
            (ButtonState::Focused, true) => &self.focused_on,
            (ButtonState::Pressed, true) => &self.pressed_on,
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
            &[ButtonSystem::NAME],
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

        for event in tgls.read_events(&mut self.tgl_rx) {
            if let Some(style) = styles.get(event.entity) {
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

        for event in styles.read_events(&mut self.style_rx) {
            if event.style.is::<ButtonStyle>() {
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
