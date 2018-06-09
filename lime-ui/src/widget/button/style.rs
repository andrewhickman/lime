use render::d2::Point;
use render::Color;
use specs::prelude::*;

use draw::{Brush, Style};
use widget::button::{Button, ButtonState, ToggleButton};

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

impl Style for ButtonStyle {
    fn draw(&self, ent: Entity, res: &Resources, visitor: &mut FnMut(&[Point], Color)) {
        let btns = ReadStorage::<Button>::fetch(res);
        if let Some(btn) = btns.get(ent) {
            self.brush(btn.state()).draw(ent, res, visitor);
        } else {
            error!("Button style applied to '{:?}' which is not a button.", ent);
        }
    }
}

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

impl Style for ToggleButtonStyle {
    fn draw(&self, ent: Entity, res: &Resources, visitor: &mut FnMut(&[Point], Color)) {
        let btns = ReadStorage::<Button>::fetch(res);
        let tgls = ReadStorage::<ToggleButton>::fetch(res);
        if let (Some(btn), Some(tgl)) = (btns.get(ent), tgls.get(ent)) {
            self.brush((btn.state(), tgl.state())).draw(ent, res, visitor);
        } else {
            error!("Toggle button style applied to '{:?}' which is not a toggle button.", ent);
        }
    }
}