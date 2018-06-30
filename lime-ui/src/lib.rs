#[macro_use]
extern crate bitflags;
extern crate cassowary;
extern crate erased_serde;
extern crate fnv;
extern crate hibitset;
extern crate lime_render as render;
extern crate lime_utils as utils;
#[macro_use]
extern crate serde_derive;
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
extern crate serde_json;

pub mod de;
pub mod draw;
pub mod event;
pub mod layout;
pub mod tree;
pub mod widget;

mod state;
#[cfg(test)]
mod tests;

pub use state::{State, StateEvent};

use shrev::EventChannel;
use specs::DispatcherBuilder;
use specs::World;

pub fn init(world: &mut World, dispatcher: &mut DispatcherBuilder<'_, '_>) {
    world.register::<State>();
    world.register::<layout::Constraints>();
    world.register::<layout::Position>();
    world.register::<tree::Node>();
    world.register::<draw::Brush>();
    world.register::<draw::Style>();
    world.register::<widget::button::Button>();
    world.register::<widget::button::ButtonStyle>();
    world.register::<widget::button::ToggleButton>();
    world.register::<widget::button::ToggleButtonStyle>();
    world.register::<widget::button::RadioButton>();
    world.register::<widget::button::RadioButtonGroup>();
    world.register::<widget::button::RadioButtonStyle>();
    world.register::<widget::grid::Grid>();

    let root = tree::Root::create(world);
    world.add_resource(event::KeyboardFocus::new(&root));
    world.add_resource(event::MouseFocus::new());
    world.add_resource(EventChannel::<event::Event>::new());

    layout::LayoutSystem::add(world, dispatcher);
    widget::button::ButtonSystem::add(world, dispatcher);
    widget::button::ToggleButtonSystem::add(world, dispatcher);
    widget::button::RadioButtonSystem::add(world, dispatcher);
    widget::button::ButtonStyleSystem::add(world, dispatcher);
    widget::button::ToggleButtonStyleSystem::add(world, dispatcher);
    widget::button::RadioButtonStyleSystem::add(world, dispatcher);
}
