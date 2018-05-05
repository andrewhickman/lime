use std::borrow::Cow;
use std::ops::DerefMut;

use render::Color;
use render::d2::{Draw, Point};
use specs::prelude::*;

pub trait Element: Draw {
    fn parent(&self) -> Option<Entity> {
        None
    }

    fn children(&self) -> Cow<[Entity]> {
        Vec::new().into()
    }
}

impl<T: ?Sized> Element for T
where
    T: DerefMut,
    T::Target: Element,
{
    fn parent(&self) -> Option<Entity> {
        self.deref().parent()
    }

    fn children(&self) -> Cow<[Entity]> {
        self.deref().children()
    }
}

pub type ElementComponent = Box<Element + Send + Sync + 'static>;

impl Component for ElementComponent {
    type Storage = DenseVecStorage<Self>;
}

impl Default for ElementComponent {
    fn default() -> Self {
        struct NullElement;

        impl Element for NullElement {}

        impl Draw for NullElement {
            fn draw(&self, _: &Resources, _: &mut FnMut(&[Point], Color)) {}
        }

        Box::new(NullElement)
    }
}

pub(crate) fn visit_children<F>(
    store: &ReadStorage<ElementComponent>,
    elem: Entity,
    visitor: &mut F,
) where
    F: FnMut(&Element),
{
    let elem: &Element = store.get(elem).unwrap();
    visitor(elem);
    for &child in elem.children().iter() {
        visit_children(store, child, visitor)
    }
}

pub(crate) fn visit_children_mut<F>(
    store: &mut WriteStorage<ElementComponent>,
    elem: Entity,
    visitor: &mut F,
) where
    F: FnMut(&mut Element),
{
    let children = {
        let elem: &mut Element = store.get_mut(elem).unwrap();
        visitor(elem);
        elem.children().into_owned()
    };

    for child in children {
        visit_children_mut(store, child, visitor)
    }
}
