use shrev::EventChannel;
use specs::prelude::*;
use specs::world::Bundle;

use {Renderer, ScreenDimensions, d2, d3};

type RenderData<'a> = (
    ReadExpect<'a, Box<d3::Draw + Send + Sync>>,
    ReadExpect<'a, Box<d2::Draw + Send + Sync>>,
    WriteExpect<'a, ScreenDimensions>,
);

impl<'a> RunNow<'a> for Renderer {
    fn run_now(&mut self, res: &'a Resources) {
        let (d3, d2, mut dim) = <RenderData as SystemData>::fetch(res);
        self.render(res, &d3, &d2, &mut dim)
    }

    fn setup(&mut self, _: &mut Resources) {}
}

impl Renderer {
    pub fn bundle<D3, D2>(&self, d3: D3, d2: D2) -> RenderBundle
    where
        D3: d3::Draw + Send + Sync + 'static,
        D2: d2::Draw + Send + Sync + 'static,
    {
        RenderBundle {
            dim: self.dimensions(),
            d2: Box::new(d2),
            d3: Box::new(d3),
        }
    }
}

pub struct RenderBundle {
    dim: ScreenDimensions,
    d3: Box<d3::Draw + Send + Sync>,
    d2: Box<d2::Draw + Send + Sync>,
}

impl Bundle for RenderBundle {
    fn add_to_world(self, world: &mut World) {
        world.add_resource(self.dim);
        world.add_resource(self.d3);
        world.add_resource(self.d2);
        world.add_resource(EventChannel::<ScreenDimensions>::new());
    }
}
