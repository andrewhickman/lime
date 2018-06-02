use render::d2::Point;
use specs::prelude::*;
use winit::{ElementState, ModifiersState, MouseButton};

use {tree, Node, Position};

#[derive(Copy, Clone, Debug)]
pub enum MouseEvent {
    Enter,
    Exit,
    Move(Point, ModifiersState),
    MoveRaw(f64, f64),
    ButtonUp(MouseButton, ModifiersState),
    ButtonDown(MouseButton, ModifiersState),
}

pub struct MouseHover {
    pub(in event) entity: Option<Entity>,
}

impl MouseHover {
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
) -> Option<Entity> {
    tree::walk_sc_rev(root, nodes, &mut |ent| {
        if let Some(pos) = poss.get(ent) {
            if pos.contains(point) {
                return Err(ent);
            }
        }
        Ok(())
    }).err()
}
