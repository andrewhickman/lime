use shrev::EventChannel;
use specs::prelude::*;
use winit::{EventsLoop, WindowBuilder};

use {d2, d3, RenderSystem, ScreenDimensions};

impl<'a, D3: d3::Draw, D2: d2::Draw> RunNow<'a> for RenderSystem<D3, D2> {
    fn run_now(&mut self, res: &'a Resources) {
        self.render(res, &mut res.fetch_mut())
    }

    fn setup(&mut self, _: &mut Resources) {}
}

pub fn init<D3, D2>(
    world: &mut World,
    events_loop: &EventsLoop,
    builder: WindowBuilder,
    d3: D3,
    d2: D2,
) -> RenderSystem<D3, D2>
where
    D3: d3::Draw + Send + Sync + 'static,
    D2: d2::Draw + Send + Sync + 'static,
{
    let render_sys = RenderSystem::new(events_loop, builder, d3, d2);
    world.add_resource(render_sys.dimensions());
    world.add_resource(EventChannel::<ScreenDimensions>::new());
    render_sys
}
