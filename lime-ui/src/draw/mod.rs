mod style;

pub use self::style::{Style, StyleEvent};

use render::d2::Renderer;
use render::Color;
use specs::prelude::*;

use layout::Position;
use tree::{self, Node, Root, WalkPostResult, WalkPreResult};
use State;

#[derive(Clone, Component, Debug, Deserialize)]
pub enum Brush {
    Color(Color),
}

impl PartialEq for Brush {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Brush::Color(lhs), Brush::Color(rhs)) => lhs == rhs,
        }
    }
}

pub struct DrawSystem;

impl DrawSystem {
    pub const NAME: &'static str = "ui::Draw";
}

impl<'a> System<'a> for DrawSystem {
    type SystemData = (
        WriteExpect<'a, Renderer>,
        ReadExpect<'a, Root>,
        ReadStorage<'a, Node>,
        ReadStorage<'a, Brush>,
        ReadStorage<'a, State>,
        ReadStorage<'a, Position>,
    );

    fn run(&mut self, (mut renderer, root, nodes, brushes, states, poss): Self::SystemData) {
        tree::walk::<(), _, _>(
            root.entity(),
            &nodes,
            &mut |ent| {
                if states.get(ent).map(State::needs_draw).unwrap_or(true) {
                    if let Some(brush) = brushes.get(ent) {
                        match *brush {
                            Brush::Color(color) => if let Some(pos) = poss.get(ent) {
                                renderer.draw_tri(&pos.tris(), color)
                            },
                        }
                    }
                }
                WalkPreResult::Continue
            },
            &mut |_| WalkPostResult::Continue,
        );
    }
}
