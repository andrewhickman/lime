use std::borrow::Cow;
use std::ops::DerefMut;

use render::d2::Draw;
use specs::prelude::*;

use Layout;

pub trait Element: Draw + Layout {
    fn children(&self) -> Cow<[Entity]> {
        Vec::new().into()
    }
}

impl<T: ?Sized> Element for T
where
    T: DerefMut,
    T::Target: Element,
{
    fn children(&self) -> Cow<[Entity]> {
        self.deref().children()
    }
}

pub type ElementComponent = Box<Element + Send + Sync + 'static>;

impl Component for ElementComponent {
    type Storage = FlaggedStorage<Self>;
}

/*
impl Default for ElementComponent {
    fn default() -> Self {
        use cassowary::Solver;
        use render::{d2::Point, Color};

        use ::layout::{Position, LayoutVars};

        struct NullElement;

        impl Element for NullElement {}

        impl Draw for NullElement {
            fn draw(&self, _: &Resources, _: &mut FnMut(&[Point], Color)) {}
        }

        impl Layout for NullElement {
            fn constraints(&self, _: &ReadStorage<LayoutVars>, _: &mut Solver) {}
            fn resize(&mut self, _: &Position) {}
        }

        Box::new(NullElement)
    }
}
*/

pub struct Root {
    pub(crate) stack: Vec<Entity>,
}

impl Root {
    pub fn new() -> Self {
        Root { stack: Vec::new() }
    }
}

pub fn add_root<E>(world: &mut World, elem: E) -> EntityBuilder
where
    E: Element + Send + Sync + 'static,
{
    let ent = world.create_entity_unchecked();
    world.write_resource::<Root>().stack.push(ent.entity);
    ent.with(Box::new(elem) as ElementComponent)
}
