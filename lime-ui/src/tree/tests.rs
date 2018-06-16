use serde_json as json;

use super::*;
use de;

#[test]
fn walk_order() {
    #[derive(Component)]
    struct Data(i32);

    let mut world = World::new();
    world.register::<Node>();
    world.register::<Data>();

    let n0 = world
        .create_entity()
        .with(Node::new())
        .with(Data(0))
        .build();
    let n1 = Node::with_parent(world.create_entity().with(Data(1)), n0).build();
    let n2 = Node::with_parent(world.create_entity().with(Data(2)), n1).build();
    let _n3 = Node::with_parent(world.create_entity().with(Data(3)), n2).build();
    let n4 = Node::with_parent(world.create_entity().with(Data(4)), n2).build();
    let _n5 = Node::with_parent(world.create_entity().with(Data(5)), n4).build();
    let _n6 = Node::with_parent(world.create_entity().with(Data(6)), n1).build();
    let n7 = Node::with_parent(world.create_entity().with(Data(7)), n1).build();
    let _n8 = Node::with_parent(world.create_entity().with(Data(8)), n7).build();
    let _n9 = Node::with_parent(world.create_entity().with(Data(9)), n7).build();
    let n10 = Node::with_parent(world.create_entity().with(Data(10)), n1).build();
    let _n11 = Node::with_parent(world.create_entity().with(Data(11)), n10).build();
    let _n12 = Node::with_parent(world.create_entity().with(Data(12)), n0).build();
    let n13 = Node::with_parent(world.create_entity().with(Data(13)), n0).build();
    let n14 = Node::with_parent(world.create_entity().with(Data(14)), n13).build();
    let _n15 = Node::with_parent(world.create_entity().with(Data(15)), n14).build();

    let comps = world.read_storage::<Data>();
    let mut expected = 0..16;
    walk(n0, &world.read_storage::<Node>(), |ent| {
        assert_eq!(comps.get(ent).unwrap().0, expected.next().unwrap());
    });

    let mut expected_rev = (0..16).rev();
    walk_rev(n0, &world.read_storage::<Node>(), |ent| {
        assert_eq!(comps.get(ent).unwrap().0, expected_rev.next().unwrap());
    });
}

#[test]
fn de() {
    const DATA: &'static str = r##"
    {
        "root": {
            "Data": 0,
            "Children": { 
                "1": {
                    "Data": 1
                },
                "2": {
                    "Data": 2,
                    "Children": {
                        "3": {
                            "Data": 3,
                            "Children": { }
                        },
                        "4": {
                            "Data": 4
                        }
                    }
                },
                "5": {
                    "Data": 5,
                    "Children": {
                        "6": {
                            "Data": 6,
                            "Children": { 
                                "7": {
                                    "Data": 7,
                                    "Children": {
                                        "8": {
                                            "Data": 8
                                        }
                                    }
                                }
                            }
                        },
                        "9": {
                            "Data": 9
                        }
                    }
                }
            }
        }
    }
    "##;

    #[derive(Component, Deserialize)]
    struct Data(i32);

    let mut world = World::new();
    let mut registry = de::Registry::new();
    registry.register::<Data>("Data");
    world.register::<Node>();
    world.register::<Data>();

    de::deserialize(
        &mut json::Deserializer::from_str(DATA),
        &registry,
        &mut world.res,
    ).unwrap();
    world.maintain();

    let data = world.read_storage::<Data>();
    let root = world.read_resource::<Root>().entity();
    let mut expected = 0..=9;
    walk(root, &world.read_storage::<Node>(), |ent| {
        assert_eq!(data.get(ent).unwrap().0, expected.next().unwrap());
    });

    let mut expected_rev = (0..=9).rev();
    walk_rev(root, &world.read_storage::<Node>(), |ent| {
        assert_eq!(data.get(ent).unwrap().0, expected_rev.next().unwrap());
    });
}