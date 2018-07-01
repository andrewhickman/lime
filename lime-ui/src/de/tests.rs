use std::borrow::Cow;
use std::io::Cursor;

use erased_serde as erased;
use serde_json as json;
use specs::prelude::*;

use super::*;

#[test]
fn de() {
    const DATA: &'static str = r#"
    {
        "root": {
            "comp1": 5,
            "comp2": {
                "value": 52,
                "name": "hello"
            }
        },
        "ent2": {
            "comp1": 6
        },
        "ent3": {
            "comp2": {
                "value": -45,
                "name": "world"
            }
        }
    }
    "#;

    #[derive(Clone, Debug, Component, Deserialize, Hash, Eq, PartialEq)]
    struct Comp1(i32);

    #[derive(Clone, Debug, Component, Deserialize, Hash, Eq, PartialEq)]
    struct Comp2 {
        value: i64,
        name: String,
    }

    let mut world = World::new();
    let mut registry = Registry::new();
    world.register::<Comp1>();
    registry.register::<Comp1>("comp1");
    world.register::<Comp2>();
    registry.register::<Comp2>("comp2");

    let mut name_map = FnvHashMap::default();

    Root::create(&mut world);
    deserialize_with_names(
        &mut json::Deserializer::from_str(DATA),
        &registry,
        &mut world.res,
        &mut name_map,
    ).unwrap();
    world.maintain();

    let ents: Vec<Entity> = (&*world.entities()).join().collect();
    assert_eq!(ents.len(), 3);

    let comp1s = world.read_storage::<Comp1>();
    let comp2s = world.read_storage::<Comp2>();

    assert_eq!(comp1s.get(name_map["root"]), Some(&Comp1(5)));
    assert_eq!(
        comp2s.get(name_map["root"]),
        Some(&Comp2 {
            value: 52,
            name: "hello".to_string(),
        })
    );

    assert_eq!(comp1s.get(name_map["ent2"]), Some(&Comp1(6)));
    assert_eq!(comp2s.get(name_map["ent2"]), None);

    assert_eq!(comp1s.get(name_map["ent3"]), None);
    assert_eq!(
        comp2s.get(name_map["ent3"]),
        Some(&Comp2 {
            value: -45,
            name: "world".to_string(),
        })
    );
}

#[test]
fn name() {
    const DATA: &'static str = r#"
    {
        "root": {
            "comp1": 5,
            "comp2": "ent2"
        },
        "ent2": {
            "comp1": 6
        },
        "ent3": {
            "comp2": "ent2"
        },
        "ent4": {
            "comp2": "root"
        }
    }
    "#;

    #[derive(Clone, Debug, Component, Deserialize, Hash, Eq, PartialEq)]
    struct Comp1(i32);

    #[derive(Clone, Debug, Component, Hash, Eq, PartialEq)]
    struct Comp2(Entity);

    impl Deserialize for Comp2 {
        fn deserialize<'de, 'a>(
            mut seed: Seed<'de, 'a>,
            deserializer: &mut erased::Deserializer<'de>,
        ) -> Result<Self, erased::Error> {
            #[derive(Deserialize)]
            struct Comp2De<'a>(#[serde(borrow)] Cow<'a, str>);

            let Comp2De(name) = <Comp2De as serde::Deserialize>::deserialize(deserializer)?;
            let entity = seed.get_entity(name)?;
            Ok(Comp2(entity))
        }
    }

    let mut world_str = World::new();
    let mut world_rdr = World::new();
    let mut registry = Registry::new();
    world_str.register::<Comp1>();
    world_rdr.register::<Comp1>();
    registry.register::<Comp1>("comp1");
    world_str.register::<Comp2>();
    world_rdr.register::<Comp2>();
    registry.register_with_deserialize::<Comp2>("comp2");

    Root::create(&mut world_str);
    Root::create(&mut world_rdr);
    let mut names_str = FnvHashMap::default();
    let mut names_rdr = FnvHashMap::default();
    deserialize_with_names(
        &mut json::Deserializer::from_str(DATA),
        &registry,
        &mut world_str.res,
        &mut names_str,
    ).unwrap();
    deserialize_with_names(
        &mut json::Deserializer::from_reader(Cursor::new(DATA)),
        &registry,
        &mut world_rdr.res,
        &mut names_rdr,
    ).unwrap();
    world_str.maintain();
    world_rdr.maintain();

    for (mut world, name_map) in vec![(world_str, names_str), (world_rdr, names_rdr)] {
        let ents: Vec<Entity> = (&*world.entities()).join().collect();
        assert_eq!(ents.len(), 4);

        let comp1s = world.read_storage::<Comp1>();
        let comp2s = world.read_storage::<Comp2>();

        assert_eq!(comp1s.get(name_map["root"]), Some(&Comp1(5)));
        assert_eq!(comp2s.get(name_map["root"]), Some(&Comp2(ents[1])));

        assert_eq!(comp1s.get(name_map["ent2"]), Some(&Comp1(6)));
        assert_eq!(comp2s.get(name_map["ent2"]), None);

        assert_eq!(comp1s.get(name_map["ent3"]), None);
        assert_eq!(comp2s.get(name_map["ent3"]), Some(&Comp2(ents[1])));

        assert_eq!(comp1s.get(name_map["ent4"]), None);
        assert_eq!(comp2s.get(name_map["ent4"]), Some(&Comp2(ents[0])));
    }
}
