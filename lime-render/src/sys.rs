use specs::prelude::*;

use ::{d2, d3, Renderer, ScreenDimensions};

pub struct RenderBundle;
pub struct RenderSystem(Renderer);

impl<'a> System<'a> for RenderSystem {
    type SystemData = (
        ReadExpect<'a, Box<d3::Draw + Send + Sync>>,
        ReadExpect<'a, Box<d2::Draw + Send + Sync>>,
        WriteExpect<'a, ScreenDimensions>,
    );

    fn run(&mut self, (d3, d2, mut dim): Self::SystemData) {
        self.0.render(&d3, &d2, &mut dim)
    }

    fn setup(&mut self, res: &mut Resources) {
        res.entry().or_insert_with(|| self.0.dimensions());
        Self::SystemData::setup(res)
    }
}
