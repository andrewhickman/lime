use std::borrow::Cow;
use std::fmt;

use erased_serde as erased;
use serde::de as serde;

use de::{Deserialize, Seed};
use widget::button::{RadioButton, RadioButtonGroup};

impl Deserialize for RadioButton {
    fn deserialize<'de, 'a>(
        mut seed: Seed<'de, 'a>,
        deserializer: &mut erased::Deserializer<'de>,
    ) -> Result<Self, erased::Error> {
        #[derive(Deserialize)]
        #[serde(rename = "RadioButtonStyle")]
        struct RadioButtonDe<'c> {
            #[serde(borrow)]
            group: Cow<'c, str>,
        }

        let RadioButtonDe { group } = serde::Deserialize::deserialize(deserializer)?;
        let group = seed.get_entity(group)?;
        Ok(RadioButton { group })
    }
}

impl Deserialize for RadioButtonGroup {
    fn deserialize<'de, 'a>(
        seed: Seed<'de, 'a>,
        deserializer: &mut erased::Deserializer<'de>,
    ) -> Result<Self, erased::Error> {
        struct Visitor<'de: 'a, 'a>(Seed<'de, 'a>);

        impl<'de: 'a, 'a> serde::Visitor<'de> for Visitor<'de, 'a> {
            type Value = RadioButtonGroup;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "sequence of entities")
            }

            fn visit_seq<A>(mut self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::SeqAccess<'de>,
            {
                let mut entities = Vec::with_capacity(seq.size_hint().unwrap_or(0));
                while let Some(name) = seq.next_element::<Cow<str>>()? {
                    entities.push(self.0.get_entity(name).map_err(serde::Error::custom)?);
                }

                Ok(RadioButtonGroup { entities })
            }
        }

        serde::Deserializer::deserialize_seq(deserializer, Visitor(seed))
    }
}
