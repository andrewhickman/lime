use specs::prelude::*;
use specs::world::Bundle;

use {Renderer, ScreenDimensions, d2, d3};

impl<'a> System<'a> for Renderer {
    type SystemData = (
        ReadExpect<'a, Box<d3::Draw + Send + Sync>>,
        ReadExpect<'a, Box<d2::Draw + Send + Sync>>,
        WriteExpect<'a, ScreenDimensions>,
    );

    fn run(&mut self, (d3, d2, mut dim): Self::SystemData) {
        self.render(&d3, &d2, &mut dim)
    }
}

impl Renderer {
    pub fn bundle(&self) -> RenderBundle {
        RenderBundle { dim: self.dimensions() }
    }
}

pub struct RenderBundle {
    dim: ScreenDimensions,
}

impl Bundle for RenderBundle {
    fn add_to_world(self, world: &mut World) {
        world.add_resource(self.dim)
    }
}