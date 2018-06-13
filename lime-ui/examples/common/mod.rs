use render::{self, d3, Color};
use specs::prelude::*;
use ui;
use ui::draw::DrawUi;
use winit::{EventsLoop, WindowBuilder};

pub struct D3;

impl d3::Draw for D3 {
    fn draw(&self, _: &Resources, _: &mut FnMut(&d3::Mesh, Color)) {}
}

pub fn init(events_loop: &EventsLoop) -> (World, Dispatcher<'static, 'static>) {
    let mut world = World::new();
    let mut dispatcher = DispatcherBuilder::new();
    let render_sys = render::init(&mut world, &events_loop, WindowBuilder::new(), D3, DrawUi);
    ui::init(&mut world, &mut dispatcher);
    let dispatcher = dispatcher.with_thread_local(render_sys).build();

    (world, dispatcher)
}
