use shrev::{EventChannel, ReaderId};
use specs::prelude::*;
use specs_mirror::StorageMutExt;
use winit::MouseButton;

use event::{Event, EventKind, MouseEvent};
use widget::button::{Button, ButtonEvent, ButtonState, RadioButton, ToggleButton,
                     ToggleButtonEvent};

pub struct ButtonSystem {
    reader: ReaderId<Event>,
}

impl ButtonSystem {
    pub const NAME: &'static str = "ui::button";

    pub(crate) fn new(world: &mut World) -> Self {
        let reader = world
            .write_resource::<EventChannel<Event>>()
            .register_reader();
        ButtonSystem { reader }
    }
}

impl<'a> System<'a> for ButtonSystem {
    type SystemData = (
        ReadExpect<'a, EventChannel<Event>>,
        WriteStorage<'a, Button>,
        WriteStorage<'a, ToggleButton>,
        ReadStorage<'a, RadioButton>,
    );

    fn run(&mut self, (events, mut btns, mut tgls, rads): Self::SystemData) {
        for &event in events.read(&mut self.reader) {
            match event.kind() {
                EventKind::Mouse(MouseEvent::Move(_, _)) => continue,
                EventKind::Mouse(MouseEvent::MoveRaw(_, _)) => continue,
                _ => (),
            };

            if let Some((btn, btn_chan)) = btns.modify(event.entity()) {
                if let ButtonState::Disabled = btn.state {
                    continue;
                }

                if let Some(rad) = rads.get(event.entity()) {
                    update_radio_button(event, btn_chan, btn, &mut tgls, rad);
                } else if let Some((tgl, tgl_chan)) = tgls.modify(event.entity()) {
                    update_toggle_button(event, btn_chan, tgl_chan, btn, tgl);
                } else {
                    update_button(event, btn_chan, btn);
                }
            }
        }
    }
}

fn update_button_common<'a>(event: Event, btn: &mut Button) -> Option<ButtonEvent> {
    use self::MouseEvent::*;

    let old = btn.state;
    let new = match event.kind() {
        EventKind::Mouse(Enter) => {
            debug_assert_eq!(old, ButtonState::Normal);
            ButtonState::Focused
        }
        EventKind::Mouse(Exit) => {
            debug_assert_ne!(old, ButtonState::Normal);
            ButtonState::Normal
        }
        EventKind::Mouse(ButtonUp(MouseButton::Left, _)) => {
            debug_assert_ne!(old, ButtonState::Normal);
            ButtonState::Focused
        }
        EventKind::Mouse(ButtonDown(MouseButton::Left, _)) => {
            debug_assert_eq!(old, ButtonState::Focused);
            ButtonState::Pressed
        }
        _ => return None,
    };

    if old != new {
        btn.state = new;
        Some(ButtonEvent {
            entity: event.entity(),
            old,
            new,
        })
    } else {
        None
    }
}

fn update_button<'a>(event: Event, btn_events: &mut EventChannel<ButtonEvent>, btn: &mut Button) {
    if let Some(btn_event) = update_button_common(event, btn) {
        btn_events.single_write(btn_event);
    }
}

fn update_toggle_button<'a>(
    event: Event,
    btn_events: &mut EventChannel<ButtonEvent>,
    tgl_events: &mut EventChannel<ToggleButtonEvent>,
    btn: &mut Button,
    tgl: &mut ToggleButton,
) {
    if let Some(btn_event) = update_button_common(event, btn) {
        if btn_event.is_press() {
            tgl.state = !tgl.state;
            tgl_events.single_write(ToggleButtonEvent {
                entity: event.entity(),
                state: tgl.state,
            });
        }
        btn_events.single_write(btn_event);
    }
}

fn update_radio_button<'a>(
    event: Event,
    btn_events: &mut EventChannel<ButtonEvent>,
    btn: &mut Button,
    tgls: &mut WriteStorage<'a, ToggleButton>,
    rad: &RadioButton,
) {
    if let Some(btn_event) = update_button_common(event, btn) {
        if btn_event.is_press() {
            for &ent in rad.group.iter() {
                if let Some((tgl, tgl_chan)) = tgls.modify(ent) {
                    let state = ent == event.entity();
                    if tgl.state != state {
                        tgl.state = state;
                        tgl_chan.single_write(ToggleButtonEvent { entity: ent, state });
                    }
                } else {
                    error!("Invalid toggle button '{:?}' in radio button group.", ent);
                }
            }
        }
        btn_events.single_write(btn_event);
    }
}
