use std::collections::HashSet;
use std::iter::FromIterator;

use serde_json::Deserializer;
use specs::prelude::*;

use super::*;

const DATA: &'static str = r#"
[
    {
        "comp1": 5,
        "comp2": {
            "value": 52,
            "name": "hello"
        }
    },
    {
        "comp1": 6
    },
    {
        "comp2": {
            "value": -45,
            "name": "world"
        }
    }
]
"#;

#[derive(Clone, Debug, Component, Deserialize, Hash, Eq, PartialEq)]
struct Comp1(i32);

#[derive(Clone, Debug, Component, Deserialize, Hash, Eq, PartialEq)]
struct Comp2 {
    value: i64,
    name: String,
}

#[test]
fn de() {
    let mut world = World::new();
    let mut registry = Registry::new();
    world.register::<Comp1>();
    registry.register::<Comp1>("comp1");
    world.register::<Comp2>();
    registry.register::<Comp2>("comp2");

    deserialize(&mut Deserializer::from_str(DATA), &registry, &world.res).unwrap();
    world.maintain();

    let comp1s: HashSet<Comp1> = (&world.read_storage::<Comp1>()).join().cloned().collect();
    assert_eq!(comp1s, HashSet::from_iter(vec![Comp1(5), Comp1(6)]));

    let comp2s: HashSet<Comp2> = (&world.read_storage::<Comp2>()).join().cloned().collect();
    assert_eq!(
        comp2s,
        HashSet::from_iter(vec![
            Comp2 {
                value: 52,
                name: "hello".to_string(),
            },
            Comp2 {
                value: -45,
                name: "world".to_string(),
            },
        ])
    );
}
