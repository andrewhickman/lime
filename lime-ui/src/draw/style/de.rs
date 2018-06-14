use std::borrow::Cow;

use erased_serde as erased;
use serde::de as serde;
use specs::prelude::*;

use de::{DeserializeAndInsert, Seed};
use draw;

impl DeserializeAndInsert for draw::Style {
    fn deserialize_and_insert<'de, 'a>(
        mut seed: Seed<'de, 'a>,
        deserializer: &mut erased::Deserializer<'de>,
    ) -> Result<(), erased::Error> {
        #[derive(Deserialize)]
        struct Style<'c> {
            #[serde(borrow)]
            style: Cow<'c, str>,
            #[serde(borrow)]
            ty: Cow<'c, str>,
        }

        let Style { style, ty } = <Style as serde::Deserialize>::deserialize(deserializer)?;
        let style = seed.get_entity(style);
        let ty = seed.reg.get_ty(ty)?;

        let res =
            draw::Style::insert_with_ty(seed.entity, style, ty, &mut WriteStorage::fetch(seed.res));
        if res.unwrap().is_some() {
            Err(serde::Error::custom(format!(
                "component defined twice for entity '{}'",
                seed.get_name(seed.entity)
            )))
        } else {
            Ok(())
        }
    }
}
