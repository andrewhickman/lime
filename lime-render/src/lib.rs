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
extern crate vulkano_win;
extern crate winit;

pub mod d2;
pub mod d3;

mod color;
mod dim;
mod render;
mod sys;

pub use self::color::Color;
pub use self::dim::ScreenDimensions;
pub use self::render::RenderSystem;
pub use self::sys::init;
