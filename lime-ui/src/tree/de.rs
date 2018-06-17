use std::borrow::Cow;
use std::fmt;

use erased_serde as erased;
use serde::de as serde;
use specs::prelude::*;

use de::{Deserialize, Seed};
use tree::Node;

impl Deserialize for Node {
    fn deserialize<'de, 'a>(
        seed: Seed<'de, 'a>,
        deserializer: &mut erased::Deserializer<'de>,
    ) -> Result<Self, erased::Error> {
        serde::DeserializeSeed::deserialize(NodeSeed(seed), deserializer)
    }
}

struct NodeSeed<'de: 'a, 'a>(Seed<'de, 'a>);

impl<'de, 'a> serde::DeserializeSeed<'de> for NodeSeed<'de, 'a> {
    type Value = Node;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor<'de: 'a, 'a>(Seed<'de, 'a>);

        impl<'de, 'a> serde::Visitor<'de> for Visitor<'de, 'a> {
            type Value = Node;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "sequence of entities")
            }

            fn visit_map<A>(mut self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::MapAccess<'de>,
            {
                let this = self.0.entity;
                let mut children = Vec::with_capacity(map.size_hint().unwrap_or(0));
                while let Some(name) = map.next_key::<Cow<str>>()? {
                    let entity = self.0.get_entity(name).map_err(serde::Error::custom)?;
                    children.push(entity);
                    map.next_value_seed(self.0.borrow().entity_seed(entity, this))?;
                }

                // Add nodes for children with no children of their own.
                let mut nodes = WriteStorage::<Node>::fetch(self.0.res);
                for &child in &children {
                    nodes.entry(child).unwrap().or_insert(Node {
                        parent: Some(this),
                        children: Vec::new(),
                    });
                }

                Ok(Node {
                    parent: self.0.parent,
                    children,
                })
            }
        }

        deserializer.deserialize_map(Visitor(self.0))
    }
}
