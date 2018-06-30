extern crate cassowary;
extern crate env_logger;
extern crate lime_render as render;
extern crate lime_ui as ui;
extern crate lime_utils as utils;
extern crate serde;
extern crate serde_json as json;
extern crate shrev;
extern crate specs;
extern crate winit;

#[allow(unused)]
mod common;

use std::panic;

use shrev::EventChannel;
use specs::prelude::*;
use ui::de::{deserialize, Registry};
use ui::draw::DrawUi;
use winit::{Event, EventsLoop, WindowBuilder, WindowEvent};

use common::D3;

pub fn init_de(events_loop: &EventsLoop, data: &str) -> (World, Dispatcher<'static, 'static>) {
    env_logger::init();
    panic::set_hook(Box::new(utils::panic_hook));

    let mut world = World::new();
    let mut dispatcher = DispatcherBuilder::new();
    let render_sys = render::init(&mut world, &events_loop, WindowBuilder::new(), D3, DrawUi);
    ui::init(&mut world, &mut dispatcher);
    deserialize(
        &mut json::Deserializer::from_str(data),
        &Registry::new(),
        &mut world.res,
    ).unwrap();
    world.maintain();
    let dispatcher = dispatcher.with_thread_local(render_sys).build();

    (world, dispatcher)
}

const DATA: &'static str = r##"
{
    "style1": {
        "RadioButtonStyle": {
            "on": {
                "disabled": { "Color": "#808080" },
                "normal": { "Color": "#FF0000" },
                "focused": { "Color": "#00FF00" },
                "pressed": { "Color": "#0000FF" }
            },
            "off": {
                "disabled": { "Color": "#808080" },
                "normal": { "Color": "#7F0000" },
                "focused": { "Color": "#007F00" },
                "pressed": { "Color": "#00007F" }
            }
        }
    },
    "root": {
        "Brush": {
            "Color": "#ABCDEF"
        },
        "Grid": {
            "rows": [
                { "type": "abs", "value": 100 },
                { "type": "rel", "value": 1 },
                { "type": "abs", "value": 100 }
            ],
            "cols": [
                { "type": "abs", "value": 100 },
                { "type": "rel", "value": 1 },
                { "type": "abs", "value": 100 },
                { "type": "rel", "value": 1 },
                { "type": "abs", "value": 100 },
                { "type": "rel", "value": 1 },
                { "type": "abs", "value": 100 }
            ]
        },
        "RadioButtonGroup": [
            "button1",
            "button2",
            "button3"
        ],
        "Children": { 
            "button1": {
                "Style": {
                    "style": "style1",
                    "ty": "RadioButtonStyle"
                },
                "Button": {
                    "state": "Normal"
                },
                "ToggleButton": {
                    "state": false
                },
                "RadioButton": {
                    "group": "root"
                },
                "Row": 1,
                "Col": 1
            },            
            "button2": {
                "Style": {
                    "style": "style1",
                    "ty": "RadioButtonStyle"
                },
                "Button": {
                    "state": "Normal"
                },
                "ToggleButton": {
                    "state": false
                },
                "RadioButton": {
                    "group": "root"
                },
                "Row": 1,
                "Col": 3
            },
            "button3": {
                "Style": {
                    "style": "style1",
                    "ty": "RadioButtonStyle"
                },
                "Button": {
                    "state": "Normal"
                },
                "ToggleButton": {
                    "state": false 
                },
                "RadioButton": {
                    "group": "root"
                },
                "Row": 1,
                "Col": 5
            }
        }
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
                ev => world.write_resource::<EventChannel<_>>().single_write(ev),
            };
        });

        dispatcher.dispatch(&world.res);
    }
}
