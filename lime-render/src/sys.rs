use shrev::EventChannel;
use specs::prelude::*;
use winit::{EventsLoop, WindowBuilder};

use {d2, d3, Renderer, ScreenDimensions};

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

pub fn init<D3, D2>(
    world: &mut World,
    events_loop: &EventsLoop,
    builder: WindowBuilder,
    d3: D3,
    d2: D2,
) -> Renderer
where
    D3: d3::Draw + Send + Sync + 'static,
    D2: d2::Draw + Send + Sync + 'static,
{
    let renderer = Renderer::new(events_loop, builder);
    world.add_resource(renderer.dimensions());
    world.add_resource::<Box<d3::Draw + Send + Sync>>(Box::new(d3));
    world.add_resource::<Box<d2::Draw + Send + Sync>>(Box::new(d2));
    world.add_resource(EventChannel::<ScreenDimensions>::new());
    renderer
}
