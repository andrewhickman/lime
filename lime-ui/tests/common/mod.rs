extern crate env_logger;

use std::panic;

use render::ScreenDimensions;
use shrev::EventChannel;
use specs::prelude::*;
use {ui, utils};

pub fn init(dims: ScreenDimensions) -> (World, Dispatcher<'static, 'static>) {
    env_logger::try_init().ok();
    panic::set_hook(Box::new(utils::panic_hook));

    let mut world = World::new();
    world.add_resource(dims);
    world.add_resource::<EventChannel<ScreenDimensions>>(Default::default());
    let (layout_sys, button_sys) = ui::init(&mut world);

    let dispatcher = DispatcherBuilder::new()
        .with(button_sys, "", &[])
        .with_thread_local(layout_sys)
        .build();

    (world, dispatcher)
}
