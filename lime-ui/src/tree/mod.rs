mod de;
#[cfg(test)]
mod tests;

use specs::prelude::*;

#[derive(Component, Debug)]
pub struct Node {
    parent: Option<Entity>,
    children: Vec<Entity>,
}

impl Node {
    pub fn add_child(node: Entity, parent: Entity, store: &mut WriteStorage<Node>) -> Self {
        store
            .entry(parent)
            .expect("invalid parent")
            .or_insert_with(Node::new)
            .children
            .push(node);
        Node {
            parent: Some(parent),
            children: Vec::new(),
        }
    }

    pub fn with_parent(builder: EntityBuilder, parent: Entity) -> EntityBuilder {
        let node = Node::add_child(builder.entity, parent, &mut builder.world.write_storage());
        builder.with(node)
    }

    fn new() -> Self {
        Node {
            parent: None,
            children: Vec::new(),
        }
    }

    pub fn parent(&self) -> Option<Entity> {
        self.parent
    }

    pub fn children(&self) -> &[Entity] {
        &self.children
    }
}

pub enum WalkPreResult<T> {
    Continue,
    Skip,
    Break(T),
}

pub enum WalkPostResult<T> {
    Continue,
    Break(T),
}

pub fn walk<T, F, G>(cur: Entity, nodes: &ReadStorage<Node>, pre: &mut F, post: &mut G) -> Option<T>
where
    F: FnMut(Entity) -> WalkPreResult<T>,
    G: FnMut(Entity) -> WalkPostResult<T>,
{
    if let Some(node) = nodes.get(cur) {
        match pre(cur) {
            WalkPreResult::Continue => (),
            WalkPreResult::Skip => return None,
            WalkPreResult::Break(val) => return Some(val),
        }

        for &ent in node.children().iter() {
            if let Some(val) = walk(ent, nodes, pre, post) {
                return Some(val);
            }
        }

        match post(cur) {
            WalkPostResult::Continue => None,
            WalkPostResult::Break(val) => Some(val),
        }
    } else {
        error!("Dead node in tree: {:?}.", cur);
        None
    }
}

pub fn walk_rev<T, F, G>(
    cur: Entity,
    nodes: &ReadStorage<Node>,
    pre: &mut F,
    post: &mut G,
) -> Option<T>
where
    F: FnMut(Entity) -> WalkPreResult<T>,
    G: FnMut(Entity) -> WalkPostResult<T>,
{
    if let Some(node) = nodes.get(cur) {
        match pre(cur) {
            WalkPreResult::Continue => (),
            WalkPreResult::Skip => return None,
            WalkPreResult::Break(val) => return Some(val),
        }

        for &ent in node.children().iter().rev() {
            if let Some(val) = walk_rev(ent, nodes, pre, post) {
                return Some(val);
            }
        }

        match post(cur) {
            WalkPostResult::Continue => None,
            WalkPostResult::Break(val) => Some(val),
        }
    } else {
        error!("Dead node in tree: {:?}.", cur);
        None
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Root {
    entity: Entity,
}

impl Root {
    pub fn create(world: &mut World) -> Self {
        let root = Root {
            entity: world.create_entity().build(),
        };
        world.add_resource(root);
        root
    }

    pub fn entity(&self) -> Entity {
        self.entity
    }
}
