use std::borrow::Cow;
use std::fmt;

use erased_serde as erased;
use render::Color;
use serde::de as serde;

use de::{DeserializeAndInsert, Seed};
use draw::Brush;

struct BrushSeed<'de: 'a, 'a>(Seed<'de, 'a>);

impl DeserializeAndInsert for Brush {
    fn deserialize_and_insert<'de, 'a>(
        seed: Seed<'de, 'a>,
        deserializer: &mut erased::Deserializer<'de>,
    ) -> Result<(), erased::Error> {
        serde::DeserializeSeed::deserialize(BrushSeed(seed), deserializer)
    }
}

impl<'de, 'a> serde::DeserializeSeed<'de> for BrushSeed<'de, 'a> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct BrushVisitor<'de: 'a, 'a>(Seed<'de, 'a>);

        impl<'de, 'a> serde::Visitor<'de> for BrushVisitor<'de, 'a> {
            type Value = ();

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "brush or style component")
            }

            fn visit_enum<A>(mut self, data: A) -> Result<Self::Value, A::Error>
            where
                A: serde::EnumAccess<'de>,
            {
                let (key, var) = data.variant::<Cow<str>>()?;
                if key == "Color" {
                    let brush = Brush::Color(serde::VariantAccess::newtype_variant::<Color>(var)?);
                    self.0.insert(brush)
                } else {
                    let de = self.0.get_deserialize_fn(key)?;
                    serde::VariantAccess::newtype_variant_seed(var, de)
                }
            }
        }

        deserializer.deserialize_enum("Brush", &[], BrushVisitor(self.0))
    }
}
