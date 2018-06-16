extern crate cassowary;
extern crate env_logger;
extern crate lime_render as render;
extern crate lime_ui as ui;
extern crate lime_utils as utils;
extern crate serde;
extern crate serde_json as json;
extern crate specs;
extern crate winit;

#[allow(unused)]
mod common;

use std::panic;

use specs::prelude::*;
use ui::de::{self, Registry};
use ui::draw::DrawUi;
use ui::event::EventSystem;
use winit::{Event, EventsLoop, WindowBuilder, WindowEvent};

use common::D3;

pub fn init_de(events_loop: &EventsLoop, data: &str) -> (World, Dispatcher<'static, 'static>) {
    env_logger::init();
    panic::set_hook(Box::new(utils::panic_hook));

    let mut world = World::new();
    let mut dispatcher = DispatcherBuilder::new();
    let render_sys = render::init(&mut world, &events_loop, WindowBuilder::new(), D3, DrawUi);
    de::init(
        &mut world,
        &mut dispatcher,
        &mut json::Deserializer::from_str(data),
        &Registry::new(),
    ).unwrap();
    let dispatcher = dispatcher.with_thread_local(render_sys).build();

    (world, dispatcher)
}

const DATA: &'static str = r##"
{
    "style1": {
        "ButtonStyle": {
            "disabled": { "Color": "#808080" },
            "normal": { "Color": "#FF0000" },
            "focused": { "Color": "#00FF00" },
            "pressed": { "Color": "#0000FF" }
        },
        "ToggleButtonStyle": {
            "disabled_on": { "Color": "#808080" },
            "normal_on": { "Color": "#FF0000" },
            "focused_on": { "Color": "#00FF00" },
            "pressed_on": { "Color": "#0000FF" },
            "disabled_off": { "Color": "#808080" },
            "normal_off": { "Color": "#FF0000" },
            "focused_off": { "Color": "#00FF00" },
            "pressed_off": { "Color": "#0000FF" }
        }
    },
    "root": {
        "Brush": {
            "Color": "#ABCDEF"
        },
        "Style": {
            "style": "style1",
            "ty": "ButtonStyle"
        },
        "Children": { }
    }
}
"##;

fn main() {
    let mut events_loop = EventsLoop::new();
    let (world, mut dispatcher) = init_de(&events_loop, DATA);

    let mut quit = false;
    while !quit {
        events_loop.poll_events(|event| {
            match event {
                Event::WindowEvent {
                    event: WindowEvent::Closed,
                    ..
                } => quit = true,
                ev => EventSystem(&ev).run_now(&world.res),
            };
        });

        dispatcher.dispatch(&world.res);
    }
}
