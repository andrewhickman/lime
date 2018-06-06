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

use shrev::EventChannel;
use specs::World;

pub fn init(world: &mut World) -> (layout::LayoutSystem, event::ButtonSystem, draw::StyleSystem) {
    world.register::<layout::Constraints>();
    world.register::<layout::Grid>();
    world.register::<layout::Position>();
    world.register::<tree::Node>();
    world.register::<draw::Brush>();
    world.register::<draw::Style>();
    world.register::<draw::Visibility>();
    world.register::<event::Button>();
    world.register::<event::ToggleButton>();
    world.register::<event::RadioButton>();

    let root = tree::Root::new(world);
    world.add_resource(event::KeyboardFocus::new(&root));
    world.add_resource(event::MouseHover::new());
    world.add_resource(root);
    world.add_resource(EventChannel::<event::Event>::new());
    let layout_sys = layout::LayoutSystem::new(world);
    let button_sys = event::ButtonSystem::new(world);
    let style_sys = draw::StyleSystem::new(world);

    (layout_sys, button_sys, style_sys)
}
