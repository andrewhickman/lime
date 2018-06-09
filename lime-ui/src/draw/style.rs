use render::d2::Point;
use render::Color;
use specs::prelude::*;

pub trait Style: Send + Sync + 'static {
    fn draw(&self, ent: Entity, res: &Resources, visitor: &mut FnMut(&[Point], Color));
}
