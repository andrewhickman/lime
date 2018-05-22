extern crate cassowary;
extern crate chan;
extern crate env_logger;
extern crate lime_render as render;
extern crate lime_ui as ui;
extern crate lime_utils as utils;
extern crate specs;
extern crate winit;

use cassowary::strength::*;
use cassowary::WeightedRelation::*;
use render::{d3, Color};
use specs::prelude::*;
use ui::elem::Root;
use ui::layout::{ConstraintUpdate, Constraints, PositionVars, ScreenVars};
use ui::{DrawUi, ElementComponent, Rect};
use winit::{Event, EventsLoop, WindowBuilder, WindowEvent};

pub struct D3;

impl d3::Draw for D3 {
    fn draw(&self, _: &Resources, _: &mut FnMut(&d3::Mesh, Color)) {}
}

#[test]
fn rect() {
    env_logger::init();
    std::panic::set_hook(Box::new(utils::hook));

    let mut events_loop = EventsLoop::new();
    let builder = WindowBuilder::new();
    let mut world = World::new();
    let mut renderer = render::init(&mut world, &events_loop, builder, D3, DrawUi);
    let layout_sys = ui::init(&mut world);

    let mut dispatcher = DispatcherBuilder::new()
        .with_thread_local(layout_sys)
        .build();

    let screen_vars: ScreenVars = *world.read_resource::<ScreenVars>();
    let vars = PositionVars::new();
    let constraints = Constraints::from_iter(
        world
            .read_resource::<chan::Sender<ConstraintUpdate>>()
            .clone(),
        vars.min_size((300.0, 500.0), STRONG).chain(vec![
            vars.left | EQ(WEAK) | 200.0,
            vars.right | EQ(WEAK) | screen_vars.width - 200.0,
            vars.top | EQ(WEAK) | 100.0,
            vars.bottom | EQ(WEAK) | screen_vars.height - 100.0,
            vars.left | EQ(REQUIRED) | screen_vars.width - vars.right,
            vars.top | EQ(REQUIRED) | screen_vars.height - vars.bottom,
        ]),
    );

    let entity = world
        .create_entity()
        .with::<ElementComponent>(Box::new(Rect::new(Color::RED)))
        .with(constraints)
        .with(vars)
        .build();

    world.write_resource::<Root>().push(entity);

    let mut quit = false;
    while !quit {
        events_loop.poll_events(|event| {
            match event {
                Event::WindowEvent {
                    event: WindowEvent::Closed,
                    ..
                } => quit = true,
                _ => (),
            };
        });

        renderer.run_now(&mut world.res);
        dispatcher.dispatch(&world.res);
    }
}
