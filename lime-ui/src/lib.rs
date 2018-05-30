extern crate cassowary;
extern crate fnv;
extern crate hibitset;
extern crate lime_render as render;
extern crate lime_utils as utils;
extern crate shrev;
extern crate specs;
#[macro_use]
extern crate log;
extern crate winit;

pub mod event;
pub mod layout;
pub mod tree;

mod draw;

pub use self::draw::{Brush, DrawUi};
pub use self::event::Event;
pub use self::layout::{Constraints, LayoutSystem, Position};
pub use self::tree::{Node, Root};

use specs::World;

pub fn init(world: &mut World) -> LayoutSystem {
    world.register::<Constraints>();
    world.register::<Position>();
    world.register::<Node>();
    world.register::<Brush>();
    world.register::<layout::Grid>();

    let root = Root::new(world);
    world.add_resource(root);
    LayoutSystem::new(world)
}
