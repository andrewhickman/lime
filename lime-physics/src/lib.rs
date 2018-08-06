extern crate fnv;
extern crate hibitset;
extern crate nphysics3d;
extern crate specs;

mod body;
mod sys;

pub use self::body::Body;

use specs::{DispatcherBuilder, World};

pub type PhysicsWorld = nphysics3d::world::World<f64>;

pub fn init(world: &mut World, dispatcher: &mut DispatcherBuilder<'_, '_>) {
    world.register::<body::Body>();

    world.add_resource(PhysicsWorld::new());

    sys::PhysicsSystem::add(world, dispatcher);
}
