use std::borrow::Cow;
use std::fmt;

use erased_serde as erased;
use serde::de as serde;
use specs::prelude::*;

use de::{DeserializeAndInsert, Seed};
use tree::Node;

impl DeserializeAndInsert for Node {
    fn deserialize_and_insert<'de, 'a>(
        mut seed: Seed<'de, 'a>,
        deserializer: &mut erased::Deserializer<'de>,
    ) -> Result<(), erased::Error> {
        {
            let mut nodes = WriteStorage::<Node>::fetch(&seed.res);
            if nodes.insert(seed.entity, Node::new()).unwrap().is_some() {
                return Err(serde::Error::custom(format!(
                    "children defined twice for entity '{}'",
                    seed.get_name(seed.entity),
                )));
            }
        }

        let children = serde::DeserializeSeed::deserialize(NodeSeed(seed.borrow()), deserializer)?;

        {
            let mut nodes = WriteStorage::<Node>::fetch(&seed.res);
            for &child in &children {
                let node = nodes.entry(child).unwrap().or_insert_with(Node::new);
                debug_assert!(node.parent.is_none());
                node.parent = Some(seed.entity);
            }

            let node = nodes.get_mut(seed.entity).unwrap();
            debug_assert!(node.children.is_empty());
            node.children = children;
        }

        Ok(())
    }
}

struct NodeSeed<'de: 'a, 'a>(Seed<'de, 'a>);

impl<'de, 'a> serde::DeserializeSeed<'de> for NodeSeed<'de, 'a> {
    type Value = Vec<Entity>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor<'de: 'a, 'a>(Seed<'de, 'a>);

        impl<'de, 'a> serde::Visitor<'de> for Visitor<'de, 'a> {
            type Value = Vec<Entity>;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "sequence of entities")
            }

            fn visit_map<A>(mut self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::MapAccess<'de>,
            {
                let mut entities = Vec::with_capacity(map.size_hint().unwrap_or(0));
                while let Some(name) = map.next_key::<Cow<str>>()? {
                    let entity = self.0.get_entity(name).map_err(serde::Error::custom)?;
                    entities.push(entity);
                    map.next_value_seed(self.0.borrow().entity_seed(entity))?;
                }
                Ok(entities)
            }
        }

        deserializer.deserialize_map(Visitor(self.0))
    }
}
