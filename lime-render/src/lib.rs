#[macro_use]
extern crate approx;
extern crate failure;
extern crate lime_utils as utils;
#[macro_use]
extern crate log;
extern crate serde;
extern crate shrev;
extern crate specs;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate vulkano;
#[macro_use]
extern crate vulkano_shader_derive;
extern crate rusttype;
extern crate vulkano_glyph;
extern crate vulkano_win;
extern crate winit;

pub mod d2;
pub mod d3;

mod color;
mod context;
mod sys;
mod target;

pub use self::color::Color;
pub use self::context::Context;
pub use self::target::{ImageTarget, SwapchainTarget, Target};

use specs::{DispatcherBuilder, World};

pub fn init<T: Target>(
    world: &mut World,
    dispatcher: &mut DispatcherBuilder,
    data: T::InitData,
    d3: &str,
    d2: &str,
) {
    sys::RenderSystem::<T>::add(world, dispatcher, data, d3, d2);
}
