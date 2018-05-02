extern crate failure;
#[macro_use]
extern crate log;
#[macro_use]
extern crate vulkano;
#[macro_use]
extern crate vulkano_shader_derive;
extern crate vulkano_win;
extern crate winit;

pub mod d2;
pub mod d3;

mod color;
mod renderer;

pub use self::color::Color;
pub use self::renderer::Renderer;

/// Utility function to panic with a payload of a failure::Error.
fn quit<F: Into<failure::Error>, T>(err: F) -> T {
    panic!(err.into())
}

fn quit_msg<S: Into<String>, T>(msg: S) -> T {
    quit(failure::err_msg(msg.into()))
}
