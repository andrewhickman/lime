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

pub struct Root {
    pub(crate) stack: Vec<Entity>,
}

impl Root {
    pub fn new() -> Self {
        Root { stack: Vec::new() }
    }

    pub fn push(&mut self, ent: Entity) {
        self.stack.push(ent)
    }
}
