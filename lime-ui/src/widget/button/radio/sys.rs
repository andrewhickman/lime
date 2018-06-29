use shrev::ReaderId;
use specs::prelude::*;
use specs_mirror::{StorageExt, StorageMutExt};

use widget::button::{
    RadioButton, RadioButtonGroup, ToggleButton, ToggleButtonEvent, ToggleButtonSystem,
};

pub struct RadioButtonSystem {
    reader: ReaderId<ToggleButtonEvent>,
    events: Vec<ToggleButtonEvent>,
}

impl RadioButtonSystem {
    pub const NAME: &'static str = "ui::RadioButton";

    pub(crate) fn add(world: &mut World, dispatcher: &mut DispatcherBuilder) {
        let reader = world.write_storage::<ToggleButton>().register_reader();
        dispatcher.add(
            RadioButtonSystem {
                reader,
                events: Vec::new(),
            },
            RadioButtonSystem::NAME,
            &[ToggleButtonSystem::NAME],
        );
    }
}

impl<'a> System<'a> for RadioButtonSystem {
    type SystemData = (
        WriteStorage<'a, ToggleButton>,
        ReadStorage<'a, RadioButton>,
        ReadStorage<'a, RadioButtonGroup>,
    );

    fn run(&mut self, (mut tgls, rads, grps): Self::SystemData) {
        self.events.extend(tgls.read_events(&mut self.reader));
        for event in self.events.drain(..) {
            if let Some(rad) = rads.get(event.entity) {
                if let Some(grp) = grps.get(rad.group) {
                    for &ent in &grp.entities {
                        if ent != event.entity {
                            if let Some((tgl, tgl_chan)) = tgls.modify(ent) {
                                tgl.set_state(ent, false, tgl_chan);
                            } else {
                                error!("Invalid toggle button '{:?}' in radio button group.", ent);
                            }
                        }
                    }
                } else {
                    error!(
                        "Invalid radio button group '{:?}' in radio button.",
                        event.entity
                    );
                }
            }
        }
        // Ignore events from this system run.
        tgls.read_events(&mut self.reader);
    }
}
