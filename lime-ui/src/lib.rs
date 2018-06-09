extern crate cassowary;
extern crate erased_serde;
extern crate fnv;
extern crate hibitset;
extern crate lime_render as render;
extern crate lime_utils as utils;
extern crate serde;
extern crate shrev;
extern crate specs;
#[macro_use]
extern crate specs_derive;
extern crate specs_mirror;
#[macro_use]
extern crate log;
extern crate winit;

#[cfg(test)]
#[macro_use]
extern crate approx;
#[cfg(test)]
extern crate env_logger;
#[cfg(test)]
#[macro_use]
extern crate serde_derive;
#[cfg(test)]
extern crate serde_json;

pub mod de;
pub mod draw;
pub mod event;
pub mod layout;
pub mod tree;
pub mod widget;

#[cfg(test)]
mod tests;

use shrev::EventChannel;
use specs::World;

pub fn init(world: &mut World) -> (layout::LayoutSystem, widget::button::ButtonSystem) {
    world.register::<layout::Constraints>();
    world.register::<layout::Position>();
    world.register::<tree::Node>();
    world.register::<draw::Brush>();
    world.register::<draw::Visibility>();
    world.register::<widget::button::Button>();
    world.register::<widget::button::ToggleButton>();
    world.register::<widget::button::RadioButton>();
    world.register::<widget::grid::Grid>();

    let root = tree::Root::new(world);
    world.add_resource(event::KeyboardFocus::new(&root));
    world.add_resource(event::MouseFocus::new());
    world.add_resource(root);
    world.add_resource(EventChannel::<event::Event>::new());
    let layout_sys = layout::LayoutSystem::new(world);
    let button_sys = widget::button::ButtonSystem::new(world);

    (layout_sys, button_sys)
}
