use std::mem;

use render::d2::{Draw, Point};
use render::Color;
use shrev::EventChannel;
use specs::prelude::*;

use {tree, Node, Position, Root};

pub enum Brush {
    Color(Color),
}

#[derive(Copy, Clone, Debug)]
pub struct Visibility {
    state: VisibilityState,
}

impl Visibility {
    pub fn new() -> Self {
        Visibility {
            state: VisibilityState::Visible,
        }
    }

    pub(crate) fn needs_draw(&self) -> bool {
        self.state == VisibilityState::Visible
    }

    pub fn get(&self) -> VisibilityState {
        self.state
    }

    pub fn set(
        &mut self,
        new: VisibilityState,
        entity: Entity,
        events: &mut EventChannel<VisibilityEvent>,
    ) {
        let old = mem::replace(&mut self.state, new);
        if old != new {
            events.single_write(VisibilityEvent { entity, old, new })
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum VisibilityState {
    Visible,
    Hidden,
    Collapsed,
}

pub struct VisibilityEvent {
    pub entity: Entity,
    pub old: VisibilityState,
    pub new: VisibilityState,
}

impl VisibilityEvent {
    pub(crate) fn needs_layout_changed(&self) -> Option<bool> {
        match (self.old, self.new) {
            (VisibilityState::Collapsed, VisibilityState::Collapsed) => None,
            (VisibilityState::Collapsed, _) => Some(true),
            (_, VisibilityState::Collapsed) => Some(false),
            (_, _) => None,
        }
    }
}

impl Component for Brush {
    type Storage = DenseVecStorage<Self>;
}

impl Component for Visibility {
    type Storage = DenseVecStorage<Self>;
}

pub struct DrawUi;

type Data<'a> = (
    ReadExpect<'a, Root>,
    Entities<'a>,
    ReadStorage<'a, Node>,
    ReadStorage<'a, Position>,
    ReadStorage<'a, Brush>,
    ReadStorage<'a, Visibility>,
);

impl Draw for DrawUi {
    fn draw(&self, res: &Resources, visitor: &mut FnMut(&[Point], Color)) {
        let (root, ents, nodes, poss, brushes, viss) = Data::fetch(res);
        let mut join = (&poss, &brushes).join();
        tree::walk(root.entity(), &nodes, |ent| {
            if viss.get(ent).map(Visibility::needs_draw).unwrap_or(true) {
                if let Some((pos, brush)) = join.get(ent, &ents) {
                    match *brush {
                        Brush::Color(color) => visitor(&pos.tris(), color),
                    }
                }
            }
        });
    }
}
