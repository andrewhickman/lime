mod parse;

use std::borrow::Cow;
use std::fmt;

use cassowary::{Constraint, Variable};
use erased_serde as erased;
use serde::de as serde;
use specs::prelude::*;

use de::{DeserializeAndInsert, Seed};
use layout::de::parse::{parse_constraint, parse_expression};
use layout::{Constraints, Position};

impl DeserializeAndInsert for Constraints {
    fn deserialize_and_insert<'de, 'a>(
        seed: Seed<'de, 'a>,
        deserializer: &mut erased::Deserializer<'de>,
    ) -> Result<(), erased::Error> {
        serde::DeserializeSeed::deserialize(ConstraintsSeed(seed), deserializer)
    }
}

struct ConstraintsSeed<'de: 'a, 'a>(Seed<'de, 'a>);

impl<'de, 'a> serde::DeserializeSeed<'de> for ConstraintsSeed<'de, 'a> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor<'de: 'a, 'a>(Seed<'de, 'a>);

        impl<'de, 'a> serde::Visitor<'de> for Visitor<'de, 'a> {
            type Value = ();

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "sequence of constraints")
            }

            fn visit_seq<A>(mut self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::SeqAccess<'de>,
            {
                let mut cons = WriteStorage::<Constraints>::fetch(self.0.res);
                let cons = cons.entry(self.0.entity).unwrap().or_insert_with(|| {
                    let mut poss = WriteStorage::<Position>::fetch(self.0.res);
                    let pos = poss.entry(self.0.entity)
                        .unwrap()
                        .or_insert_with(Default::default);
                    Constraints::new(pos)
                });

                if let Some(size) = seq.size_hint() {
                    cons.reserve(size);
                }

                while let Some(con) = seq.next_element_seed(ConstraintSeed(self.0.borrow()))? {
                    cons.add(con);
                }

                Ok(())
            }
        }

        deserializer.deserialize_seq(Visitor(self.0))
    }
}

struct ConstraintSeed<'de: 'a, 'a>(Seed<'de, 'a>);

impl<'de, 'a> serde::DeserializeSeed<'de> for ConstraintSeed<'de, 'a> {
    type Value = Constraint;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let src = <Cow<str> as serde::Deserialize>::deserialize(deserializer)?;
        parse_constraint(self.0, src).map_err(serde::Error::custom)
    }
}

impl DeserializeAndInsert for Position {
    fn deserialize_and_insert<'de, 'a>(
        seed: Seed<'de, 'a>,
        deserializer: &mut erased::Deserializer<'de>,
    ) -> Result<(), erased::Error> {
        serde::DeserializeSeed::deserialize(PositionSeed(seed), deserializer)
    }
}

struct PositionSeed<'de: 'a, 'a>(Seed<'de, 'a>);

impl<'de, 'a> serde::DeserializeSeed<'de> for PositionSeed<'de, 'a> {
    type Value = ();

    fn deserialize<D>(mut self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename = "Position")]
        struct PositionDe<'c> {
            #[serde(borrow, default)]
            left: Option<Cow<'c, str>>,
            #[serde(borrow, default)]
            top: Option<Cow<'c, str>>,
            #[serde(borrow, default)]
            right: Option<Cow<'c, str>>,
            #[serde(borrow, default)]
            bottom: Option<Cow<'c, str>>,
        }

        let PositionDe {
            left,
            top,
            right,
            bottom,
        } = serde::Deserialize::deserialize(deserializer)?;

        let pos = {
            let mut poss = WriteStorage::<Position>::fetch(self.0.res);
            poss.entry(self.0.entity)
                .unwrap()
                .or_insert_with(Default::default)
                .clone()
        };

        let left = parse_position_constraint(self.0.borrow(), pos.left_var(), left)?;
        let top = parse_position_constraint(self.0.borrow(), pos.top_var(), top)?;
        let right = parse_position_constraint(self.0.borrow(), pos.right_var(), right)?;
        let bottom = parse_position_constraint(self.0.borrow(), pos.bottom_var(), bottom)?;

        let mut cons = WriteStorage::<Constraints>::fetch(self.0.res);
        cons.entry(self.0.entity)
            .unwrap()
            .or_insert_with(|| Constraints::new(&pos))
            .extend(left.into_iter().chain(top).chain(right).chain(bottom));

        Ok(())
    }
}

fn parse_position_constraint<'de, 'a, E>(
    seed: Seed<'de, 'a>,
    v: Variable,
    s: Option<Cow<'de, str>>,
) -> Result<Option<Constraint>, E>
where
    E: serde::Error,
{
    use cassowary::strength::REQUIRED;
    use cassowary::WeightedRelation::EQ;

    // Using REQUIRED constraints here should be OK since there is no way to create an
    // unsolvable system without specifying a REQUIRED constraint elsewhere.
    Ok(if let Some(s) = s {
        Some(v | EQ(REQUIRED) | parse_expression(seed, s).map_err(serde::Error::custom)?)
    } else {
        None
    })
}
