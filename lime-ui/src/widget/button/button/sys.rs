use shrev::{EventChannel, ReaderId};
use specs::prelude::*;
use specs_mirror::StorageMutExt;
use winit::MouseButton;

use event::{Event, EventKind, MouseEvent};
use widget::button::{Button, ButtonState};

pub struct ButtonSystem {
    reader: ReaderId<Event>,
}

impl ButtonSystem {
    pub const NAME: &'static str = "ui::Button";

    pub(crate) fn add(world: &mut World, dispatcher: &mut DispatcherBuilder) {
        let reader = world
            .write_resource::<EventChannel<Event>>()
            .register_reader();
        dispatcher.add(ButtonSystem { reader }, ButtonSystem::NAME, &[]);
    }
}

impl<'a> System<'a> for ButtonSystem {
    type SystemData = (
        ReadExpect<'a, EventChannel<Event>>,
        WriteStorage<'a, Button>,
    );

    fn run(&mut self, (events, mut btns): Self::SystemData) {
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

                let state = match event.kind() {
                    EventKind::Mouse(MouseEvent::Enter) => {
                        debug_assert_eq!(btn.state, ButtonState::Normal);
                        ButtonState::Focused
                    }
                    EventKind::Mouse(MouseEvent::Exit) => {
                        debug_assert_ne!(btn.state, ButtonState::Normal);
                        ButtonState::Normal
                    }
                    EventKind::Mouse(MouseEvent::ButtonUp(MouseButton::Left, _)) => {
                        debug_assert_ne!(btn.state, ButtonState::Normal);
                        ButtonState::Focused
                    }
                    EventKind::Mouse(MouseEvent::ButtonDown(MouseButton::Left, _)) => {
                        debug_assert_eq!(btn.state, ButtonState::Focused);
                        ButtonState::Pressed
                    }
                    _ => continue,
                };

                btn.set_state(event.entity(), state, btn_chan);
            }
        }
    }
}
