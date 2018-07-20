use env_logger;
use render;
use shrev::EventChannel;
use specs::prelude::*;
use ui;
use ui::draw::DrawSystem;
use utils::{self, throw};
use winit::{Event, EventsLoop, WindowBuilder};

pub struct D3;

impl D3 {
    pub const NAME: &'static str = "D3";
}

impl<'a> System<'a> for D3 {
    type SystemData = ();

    fn run(&mut self, (): Self::SystemData) {}
}

pub fn init(events_loop: &EventsLoop) -> (World, Dispatcher<'static, 'static>) {
    env_logger::init();
    utils::set_panic_hook();

    let mut world = World::new();
    let mut dispatcher =
        DispatcherBuilder::new()
            .with(D3, D3::NAME, &[])
            .with(DrawSystem, DrawSystem::NAME, &[]);

    let window = WindowBuilder::new()
        .build(events_loop)
        .unwrap_or_else(throw);
    world.add_resource(EventChannel::<Event>::new());
    render::init(
        &mut world,
        &mut dispatcher,
        window,
        D3::NAME,
        DrawSystem::NAME,
    );
    ui::init(&mut world, &mut dispatcher);

    (world, dispatcher.build())
}
