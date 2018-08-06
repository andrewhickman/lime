mod store;

pub use self::store::BodyStorage;

use nphysics3d::object::BodyHandle;
use specs::prelude::*;

pub struct Body {
    handle: BodyHandle,
}

impl Component for Body {
    type Storage = BodyStorage;
}
