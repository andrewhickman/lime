use render::d2::Point;
use specs::prelude::*;
use winit::{ElementState, ModifiersState, MouseButton};

use layout::Position;
use tree::{self, Node, WalkPostResult, WalkPreResult};
use State;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MouseEvent {
    Enter,
    Exit,
    Move(Point, ModifiersState),
    MoveRaw(f64, f64),
    ButtonUp(MouseButton, ModifiersState),
    ButtonDown(MouseButton, ModifiersState),
}

pub struct MouseFocus {
    pub(in event) entity: Option<Entity>,
    pub(in event) point: Point,
}

impl MouseFocus {
    pub(crate) fn new() -> Self {
        MouseFocus {
            entity: None,
            point: Point::origin(),
        }
    }

    pub fn entity(&self) -> Option<Entity> {
        self.entity
    }
}

impl MouseEvent {
    pub(in event) fn from_input(
        state: ElementState,
        button: MouseButton,
        modifiers: ModifiersState,
    ) -> Self {
        match state {
            ElementState::Pressed => MouseEvent::ButtonDown(button, modifiers),
            ElementState::Released => MouseEvent::ButtonUp(button, modifiers),
        }
    }
}

pub(in event) fn hit_test(
    root: Entity,
    point: Point,
    nodes: &ReadStorage<Node>,
    poss: &ReadStorage<Position>,
    states: &ReadStorage<State>,
) -> Option<Entity> {
    tree::walk_rev::<Entity, _, _>(root, nodes, &mut |_| WalkPreResult::Continue, &mut |ent| {
        if states.get(ent).map(State::needs_draw).unwrap_or(true) {
            if let Some(pos) = poss.get(ent) {
                if pos.contains(point) {
                    return WalkPostResult::Break(ent);
                }
            }
        }
        WalkPostResult::Continue
    })
}
