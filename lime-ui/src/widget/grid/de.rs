use erased_serde as erased;
use serde::de as serde;
use specs::prelude::*;

use de::{Deserialize, Insert, Seed};
use layout::{Constraints, Position};
use widget::grid::{Grid, Size};

impl Deserialize for Grid {
    fn deserialize<'de, 'a>(
        seed: Seed<'de, 'a>,
        deserializer: &mut erased::Deserializer<'de>,
    ) -> Result<Self, erased::Error> {
        serde::DeserializeSeed::deserialize(GridSeed(seed), deserializer)
    }
}

struct GridSeed<'de: 'a, 'a>(Seed<'de, 'a>);

impl<'de, 'a> serde::DeserializeSeed<'de> for GridSeed<'de, 'a> {
    type Value = Grid;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename = "Grid")]
        struct GridDe {
            rows: Vec<Size>,
            cols: Vec<Size>,
        }

        let GridDe { rows, cols } = serde::Deserialize::deserialize(deserializer)?;

        let mut poss = WriteStorage::<Position>::fetch(self.0.res);
        let pos = poss.entry(self.0.entity)
            .unwrap()
            .or_insert_with(Default::default);

        let mut cons = WriteStorage::<Constraints>::fetch(self.0.res);
        let con = cons.entry(self.0.entity)
            .unwrap()
            .or_insert_with(|| Constraints::new(pos));

        Ok(Grid::new(pos, con, cols, rows))
    }
}

#[derive(Deserialize)]
pub(crate) struct Row(u32);

impl Insert for Row {
    fn insert<'de, 'a>(self, seed: Seed<'de, 'a>) -> Result<Option<Self>, erased::Error> {
        let grids = ReadStorage::<Grid>::fetch(seed.res);
        if let Some(grid) = seed.parent.and_then(|ent| grids.get(ent)) {
            let mut poss = WriteStorage::<Position>::fetch(seed.res);
            let pos = poss.entry(seed.entity)
                .unwrap()
                .or_insert_with(Default::default);

            let mut cons = WriteStorage::<Constraints>::fetch(seed.res);
            let con = cons.entry(seed.entity)
                .unwrap()
                .or_insert_with(|| Constraints::new(pos));

            grid.insert_row(self.0, pos, con);
            Ok(None)
        } else {
            Err(serde::Error::custom(format!(
                "row defined on entity '{}' which is not a child of a grid",
                seed.get_name(seed.entity)
            )))
        }
    }
}

#[derive(Deserialize)]
pub(crate) struct Col(u32);

impl Insert for Col {
    fn insert<'de, 'a>(self, seed: Seed<'de, 'a>) -> Result<Option<Self>, erased::Error> {
        let grids = ReadStorage::<Grid>::fetch(seed.res);
        if let Some(grid) = seed.parent.and_then(|ent| grids.get(ent)) {
            let mut poss = WriteStorage::<Position>::fetch(seed.res);
            let pos = poss.entry(seed.entity)
                .unwrap()
                .or_insert_with(Default::default);

            let mut cons = WriteStorage::<Constraints>::fetch(seed.res);
            let con = cons.entry(seed.entity)
                .unwrap()
                .or_insert_with(|| Constraints::new(pos));

            grid.insert_col(self.0, pos, con);
            Ok(None)
        } else {
            Err(serde::Error::custom(format!(
                "column defined on entity '{}' which is not a child of a grid",
                seed.get_name(seed.entity)
            )))
        }
    }
}
