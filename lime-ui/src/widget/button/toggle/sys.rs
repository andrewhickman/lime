use shrev::ReaderId;
use specs::prelude::*;
use specs_mirror::{StorageExt, StorageMutExt};

use widget::button::{Button, ButtonEvent, ButtonSystem, RadioButton, ToggleButton};

pub struct ToggleButtonSystem {
    reader: ReaderId<ButtonEvent>,
}

impl ToggleButtonSystem {
    pub const NAME: &'static str = "ui::ToggleButton";

    pub(crate) fn add(world: &mut World, dispatcher: &mut DispatcherBuilder) {
        let reader = world.write_storage::<Button>().register_reader();
        dispatcher.add(
            ToggleButtonSystem { reader },
            ToggleButtonSystem::NAME,
            &[ButtonSystem::NAME],
        );
    }
}

impl<'a> System<'a> for ToggleButtonSystem {
    type SystemData = (
        ReadStorage<'a, Button>,
        WriteStorage<'a, ToggleButton>,
        ReadStorage<'a, RadioButton>,
    );

    fn run(&mut self, (btns, mut tgls, rads): Self::SystemData) {
        for &event in btns.read_events(&mut self.reader) {
            if event.is_press() {
                if let Some((tgl, tgl_chan)) = tgls.modify(event.entity) {
                    let state = !tgl.state();
                    if state || !rads.contains(event.entity) {
                        tgl.set_state(event.entity, state, tgl_chan);
                    }
                }
            }
        }
    }
}
