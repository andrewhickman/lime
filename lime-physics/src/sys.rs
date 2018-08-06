use specs::prelude::*;

use body::{Body, BodyStorage};
use PhysicsWorld;

pub struct PhysicsSystem;

impl PhysicsSystem {
    pub const NAME: &'static str = "physics::Physics";

    pub(crate) fn add(_world: &mut World, dispatcher: &mut DispatcherBuilder<'_, '_>) {
        dispatcher.add(PhysicsSystem, PhysicsSystem::NAME, &[]);
    }
}

impl<'a> System<'a> for PhysicsSystem {
    type SystemData = (WriteExpect<'a, PhysicsWorld>, WriteStorage<'a, Body>);

    fn run(&mut self, (mut world, mut bodies): Self::SystemData) {
        BodyStorage::handle_removed(&mut bodies, |removed| world.remove_bodies(removed));
        world.step();
    }
}
