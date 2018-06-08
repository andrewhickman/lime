use std::panic;

use render::ScreenDimensions;
use shrev::EventChannel;
use specs::prelude::*;

use super::*;
use draw::StyleSystem;
use widget::button::ButtonSystem;

pub fn init_test(dims: ScreenDimensions) -> (World, Dispatcher<'static, 'static>) {
    env_logger::try_init().ok();
    panic::set_hook(Box::new(utils::panic_hook));

    let mut world = World::new();
    world.add_resource(dims);
    world.add_resource::<EventChannel<ScreenDimensions>>(Default::default());
    let (layout_sys, button_sys, style_sys) = init(&mut world);

    let dispatcher = DispatcherBuilder::new()
        .with(button_sys, ButtonSystem::NAME, &[])
        .with(style_sys, StyleSystem::NAME, &[])
        .with_thread_local(layout_sys)
        .build();

    (world, dispatcher)
}
