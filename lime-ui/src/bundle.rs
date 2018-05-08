use specs::prelude::*;
use specs::world;

use elem;

pub struct Bundle {
    root: elem::Root,
}

impl Bundle {
    pub fn new() -> Self {
        Bundle {
            root: elem::Root::new(),
        }
    }
}

impl world::Bundle for Bundle {
    fn add_to_world(self, world: &mut World) {
        world.add_resource(self.root);
        world.register::<elem::ElementComponent>();
    }
}
