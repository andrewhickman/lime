extern crate failure;
#[macro_use]
extern crate log;
extern crate shrev;
extern crate specs;
#[macro_use]
extern crate vulkano;
#[macro_use]
extern crate vulkano_shader_derive;
extern crate vulkano_win;
extern crate winit;

pub mod d2;
pub mod d3;

mod color;
mod render;
mod sys;

pub use self::color::Color;
pub use self::render::Renderer;
pub use self::sys::RenderBundle;

#[derive(Copy, Clone, Debug)]
/// Resource holding the current size of the screen.
pub struct ScreenDimensions {
    pub w: u32,
    pub h: u32,
}

impl Into<[u32; 2]> for ScreenDimensions {
    fn into(self) -> [u32; 2] {
        [self.w, self.h]
    }
}

impl From<[u32; 2]> for ScreenDimensions {
    fn from(arr: [u32; 2]) -> Self {
        ScreenDimensions {
            w: arr[0],
            h: arr[1],
        }
    }
}

/// Utility function to panic with a payload of a failure::Error.
fn quit<F: Into<failure::Error>, T>(err: F) -> T {
    panic!(err.into())
}

fn quit_msg<S: Into<String>, T>(msg: S) -> T {
    quit(failure::err_msg(msg.into()))
}
