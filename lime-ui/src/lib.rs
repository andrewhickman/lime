extern crate cassowary;
extern crate fnv;
extern crate hibitset;
extern crate lime_render as render;
extern crate lime_utils as utils;
extern crate shrev;
extern crate specs;
#[macro_use]
extern crate log;
#[macro_use]
extern crate specs_derive;

mod draw;
mod layout;
mod tree;

pub use self::draw::{Brush, DrawUi};
pub use self::layout::{Constraints, LayoutSystem, Position};
pub use self::tree::{Node, Root};

use specs::World;

pub fn init(world: &mut World) -> LayoutSystem {
    world.register::<Constraints>();
    world.register::<Position>();
    world.register::<Node>();
    world.register::<Brush>();

    let root = Root::new(world);
    world.add_resource(root);
    LayoutSystem::new(world)
}
