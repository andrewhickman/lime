use shrev::EventChannel;
use specs::prelude::*;

use super::*;

pub fn init_test() -> (World, Dispatcher<'static, 'static>) {
    env_logger::try_init().ok();

    let mut world = World::new();
    world.add_resource(EventChannel::<winit::Event>::new());
    let mut dispatcher = DispatcherBuilder::new();
    init(&mut world, &mut dispatcher);

    (world, dispatcher.build())
}
