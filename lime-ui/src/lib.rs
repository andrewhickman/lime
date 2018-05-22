pub extern crate cassowary;

#[macro_use]
extern crate chan;
extern crate lime_render as render;
extern crate lime_utils as utils;
extern crate shrev;
extern crate specs;
#[macro_use]
extern crate log;
#[macro_use]
extern crate specs_derive;

pub mod elem;
pub mod layout;

mod draw;
mod rect;

pub use self::draw::DrawUi;
pub use self::elem::{Element, ElementComponent};
pub use self::layout::{Layout, LayoutSystem, Resize};
pub use self::rect::Rect;

pub mod prelude {
    pub use cassowary::{self, Constraint, Variable};
}

use specs::World;

use elem::Root;
use layout::{ConstraintUpdate, Constraints, PositionVars, ScreenVars};

pub fn init(world: &mut World) -> LayoutSystem {
    let (con_tx, con_rx) = chan::async();
    world.add_resource::<Root>(Root::new());
    world.add_resource::<ScreenVars>(ScreenVars::new());
    world.add_resource::<chan::Sender<ConstraintUpdate>>(con_tx);
    world.register::<ElementComponent>();
    world.register::<Constraints>();
    world.register::<PositionVars>();
    LayoutSystem::new(&mut world.res, con_rx)
}
