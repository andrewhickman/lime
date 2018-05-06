use std::borrow::Cow;
use std::ops::DerefMut;

use render::d2::Draw;
use specs::prelude::*;

use {layout::LayoutVars, Layout};

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
    type Storage = DenseVecStorage<Self>;
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

pub fn add<E>(world: &mut World, elem: E) -> EntityBuilder
where
    E: Element + Send + Sync + 'static,
{
    world
        .create_entity()
        .with(Box::new(elem) as ElementComponent)
        .with(LayoutVars::new())
}

pub fn add_root<E>(world: &mut World, elem: E) -> EntityBuilder
where
    E: Element + Send + Sync + 'static,
{
    let ent = world.create_entity_unchecked();
    world.write_resource::<Root>().stack.push(ent.entity);
    ent.with(Box::new(elem) as ElementComponent)
        .with(LayoutVars::new())
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

/*
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
*/
