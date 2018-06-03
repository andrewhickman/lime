use render::{self, d3, Color};
use specs::prelude::*;
use ui::{self, DrawUi};
use winit::{EventsLoop, WindowBuilder};

pub struct D3;

impl d3::Draw for D3 {
    fn draw(&self, _: &Resources, _: &mut FnMut(&d3::Mesh, Color)) {}
}

pub fn init(events_loop: &EventsLoop) -> (World, Dispatcher<'static, 'static>) {
    let mut world = World::new();
    let renderer = render::init(&mut world, &events_loop, WindowBuilder::new(), D3, DrawUi);
    let (layout_sys, button_sys) = ui::init(&mut world);

    let dispatcher = DispatcherBuilder::new()
        .with(button_sys, "", &[])
        .with_thread_local(layout_sys)
        .with_thread_local(renderer)
        .build();

    (world, dispatcher)
}
