use specs::prelude::*;

#[derive(Component, Debug)]
pub struct Node {
    parent: Option<Entity>,
    children: Vec<Entity>,
}

impl Node {
    pub fn add_child(node: Entity, parent: Entity, store: &mut WriteStorage<Node>) -> Self {
        store
            .get_mut(parent)
            .expect("Invalid parent.")
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

    fn root() -> Self {
        Node {
            parent: None,
            children: Vec::new(),
        }
    }
}

pub fn walk<F>(cur: Entity, nodes: &ReadStorage<Node>, visit: &mut F)
where
    F: FnMut(Entity),
{
    if let Some(node) = nodes.get(cur) {
        visit(cur);
        for &ent in &node.children {
            walk(ent, nodes, visit)
        }
    } else {
        error!("Dead node in tree: {:?}.", cur);
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Root {
    ent: Entity,
}

impl Root {
    pub(crate) fn new(world: &mut World) -> Self {
        Root {
            ent: world.create_entity().with(Node::root()).build(),
        }
    }

    pub fn entity(&self) -> Entity {
        self.ent
    }
}

#[test]
fn test_tree() {
    #[derive(Component)]
    struct Comp(i32);

    let mut world = World::new();
    world.register::<Node>();
    world.register::<Comp>();

    let n0 = world
        .create_entity()
        .with(Node::root())
        .with(Comp(0))
        .build();
    let n1 = Node::with_parent(world.create_entity().with(Comp(1)), n0).build();
    let n2 = Node::with_parent(world.create_entity().with(Comp(2)), n1).build();
    let _n3 = Node::with_parent(world.create_entity().with(Comp(3)), n2).build();
    let n4 = Node::with_parent(world.create_entity().with(Comp(4)), n2).build();
    let _n5 = Node::with_parent(world.create_entity().with(Comp(5)), n4).build();
    let _n6 = Node::with_parent(world.create_entity().with(Comp(6)), n1).build();
    let n7 = Node::with_parent(world.create_entity().with(Comp(7)), n1).build();
    let _n8 = Node::with_parent(world.create_entity().with(Comp(8)), n7).build();
    let _n9 = Node::with_parent(world.create_entity().with(Comp(9)), n7).build();
    let n10 = Node::with_parent(world.create_entity().with(Comp(10)), n1).build();
    let _n11 = Node::with_parent(world.create_entity().with(Comp(11)), n10).build();
    let _n12 = Node::with_parent(world.create_entity().with(Comp(12)), n0).build();
    let n13 = Node::with_parent(world.create_entity().with(Comp(13)), n0).build();
    let n14 = Node::with_parent(world.create_entity().with(Comp(14)), n13).build();
    let _n15 = Node::with_parent(world.create_entity().with(Comp(15)), n14).build();

    let comps = world.read_storage::<Comp>();
    let mut expected = 0..16;
    walk(n0, &world.read_storage::<Node>(), &mut |ent| {
        assert_eq!(comps.get(ent).unwrap().0, expected.next().unwrap());
    });
}
