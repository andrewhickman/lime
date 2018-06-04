extern crate cassowary;
extern crate fnv;
extern crate hibitset;
extern crate lime_render as render;
extern crate lime_utils as utils;
extern crate shrev;
extern crate specs;
extern crate specs_mirror;
#[macro_use]
extern crate log;
extern crate winit;

pub mod draw;
pub mod event;
pub mod layout;
pub mod tree;

pub use self::draw::{Brush, DrawUi};
pub use self::event::{Event, EventKind, EventSystem, KeyboardEvent, MouseEvent};
pub use self::layout::{Constraints, LayoutSystem, Position};
pub use self::tree::{Node, Root};

use shrev::EventChannel;
use specs::World;

pub fn init(world: &mut World) -> (LayoutSystem, event::ButtonSystem) {
    world.register::<Constraints>();
    world.register::<Position>();
    world.register::<Node>();
    world.register::<Brush>();
    world.register::<draw::Visibility>();
    world.register::<layout::Grid>();
    world.register::<event::Button>();
    world.register::<event::ToggleButton>();
    world.register::<event::RadioButton>();

    let root = Root::new(world);
    world.add_resource(event::KeyboardFocus::new(&root));
    world.add_resource(event::MouseHover::new());
    world.add_resource(root);
    world.add_resource(EventChannel::<Event>::new());
    world.add_resource(EventChannel::<draw::VisibilityEvent>::new());
    world.add_resource(EventChannel::<event::ButtonEvent>::new());
    world.add_resource(EventChannel::<event::ToggleButtonEvent>::new());
    let layout_sys = LayoutSystem::new(world);
    let button_sys = event::ButtonSystem::new(world);

    (layout_sys, button_sys)
}
