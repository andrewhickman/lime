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

use shrev::EventChannel;
use ui::de::{deserialize, Registry};
use winit::{Event, EventsLoop, WindowEvent};

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
    let (mut world, mut dispatcher) = common::init(&events_loop);

    deserialize(
        &mut json::Deserializer::from_str(DATA),
        &Registry::new(),
        &mut world.res,
    ).unwrap();
    world.maintain();

    let mut quit = false;
    while !quit {
        events_loop.poll_events(|event| {
            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => quit = true,
                ev => world.write_resource::<EventChannel<_>>().single_write(ev),
            };
        });

        dispatcher.dispatch(&world.res);
    }
}
