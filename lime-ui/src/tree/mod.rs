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
}

pub fn walk<F>(cur: Entity, nodes: &ReadStorage<Node>, mut visit: F)
where
    F: FnMut(Entity),
{
    walk_sc::<(), _>(cur, nodes, &mut |ent| Ok(visit(ent))).unwrap()
}

pub fn walk_sc<E, F>(cur: Entity, nodes: &ReadStorage<Node>, visit: &mut F) -> Result<(), E>
where
    F: FnMut(Entity) -> Result<(), E>,
{
    if let Some(node) = nodes.get(cur) {
        visit(cur)?;
        for &ent in &node.children {
            walk_sc(ent, nodes, visit)?;
        }
    } else {
        error!("Dead node in tree: {:?}.", cur);
    }
    Ok(())
}

pub fn walk_rev<F>(cur: Entity, nodes: &ReadStorage<Node>, mut visit: F)
where
    F: FnMut(Entity),
{
    walk_sc_rev::<(), _>(cur, nodes, &mut |ent| Ok(visit(ent))).unwrap()
}

pub fn walk_sc_rev<E, F>(cur: Entity, nodes: &ReadStorage<Node>, visit: &mut F) -> Result<(), E>
where
    F: FnMut(Entity) -> Result<(), E>,
{
    if let Some(node) = nodes.get(cur) {
        for &ent in node.children.iter().rev() {
            walk_sc_rev(ent, nodes, visit)?;
        }
        visit(cur)?;
    } else {
        error!("Dead node in tree: {:?}.", cur);
    }
    Ok(())
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
