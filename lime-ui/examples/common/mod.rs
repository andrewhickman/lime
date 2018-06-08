use render::{self, d3, Color};
use specs::prelude::*;
use ui;
use ui::draw::{DrawUi, StyleSystem};
use ui::widget::button::ButtonSystem;
use winit::{EventsLoop, WindowBuilder};

pub struct D3;

impl d3::Draw for D3 {
    fn draw(&self, _: &Resources, _: &mut FnMut(&d3::Mesh, Color)) {}
}

pub fn init(events_loop: &EventsLoop) -> (World, Dispatcher<'static, 'static>) {
    let mut world = World::new();
    let render_sys = render::init(&mut world, &events_loop, WindowBuilder::new(), D3, DrawUi);
    let (layout_sys, button_sys, style_sys) = ui::init(&mut world);

    let dispatcher = DispatcherBuilder::new()
        .with(button_sys, ButtonSystem::NAME, &[])
        .with(style_sys, StyleSystem::NAME, &[ButtonSystem::NAME])
        .with_thread_local(layout_sys)
        .with_thread_local(render_sys)
        .build();

    (world, dispatcher)
}
