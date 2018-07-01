use fnv::FnvHashMap;
use render::Color;
use serde_json as json;
use specs::prelude::*;

use super::*;
use de;
use draw::Brush;
use tree::Root;
use widget::button::{ButtonStyle, RadioButtonStyle, ToggleButtonStyle};

#[test]
fn de() {
    const DATA: &'static str = r##"
    {
        "root": {
            "Brush": {
                "Color": "#FF0000"
            },
            "Style": {
                "style": "style1",
                "ty": "ButtonStyle"
            }
        },
        "ent2": {
            "Style": {
                "style": "style1",
                "ty": "ToggleButtonStyle"
            }
        },
        "style1": {
            "ButtonStyle": {
                "disabled": { "Color": "#808080" },
                "normal": { "Color": "#FF0000" },
                "focused": { "Color": "#00FF00" },
                "pressed": { "Color": "#0000FF" }
            },
            "ToggleButtonStyle": {
                "on": {
                    "disabled": { "Color": "#808080" },
                    "normal": { "Color": "#FF0000" },
                    "focused": { "Color": "#00FF00" },
                    "pressed": { "Color": "#0000FF" }
                },
                "off": {
                    "disabled": { "Color": "#808080" },
                    "normal": { "Color": "#FF0000" },
                    "focused": { "Color": "#00FF00" },
                    "pressed": { "Color": "#0000FF" }
                }
            },
            "RadioButtonStyle": {
                "on": {
                    "disabled": { "Color": "#808080" },
                    "normal": { "Color": "#FF0000" },
                    "focused": { "Color": "#00FF00" },
                    "pressed": { "Color": "#0000FF" }
                },
                "off": {
                    "disabled": { "Color": "#808080" },
                    "normal": { "Color": "#FF0000" },
                    "focused": { "Color": "#00FF00" },
                    "pressed": { "Color": "#0000FF" }
                }
            }
        }
    }
    "##;

    let mut world = World::new();
    let registry = de::Registry::new();
    world.register::<Brush>();
    world.register::<Style>();
    world.register::<ButtonStyle>();
    world.register::<ToggleButtonStyle>();
    world.register::<RadioButtonStyle>();

    let mut name_map = FnvHashMap::default();

    Root::create(&mut world);
    de::deserialize_with_names(
        &mut json::Deserializer::from_str(DATA),
        &registry,
        &mut world.res,
        &mut name_map,
    ).unwrap();
    world.maintain();

    let ents: Vec<Entity> = (&*world.entities()).join().collect();
    assert_eq!(ents.len(), 3);

    let styles = world.read_storage::<Style>();
    assert_eq!(
        styles.get(name_map["root"]).unwrap().get(),
        name_map["style1"]
    );
    assert!(styles.get(name_map["root"]).unwrap().is::<ButtonStyle>());
    assert!(styles.get(name_map["style1"]).is_none());
    assert_eq!(
        styles.get(name_map["ent2"]).unwrap().get(),
        name_map["style1"]
    );
    assert!(
        styles
            .get(name_map["ent2"])
            .unwrap()
            .is::<ToggleButtonStyle>()
    );

    let brushes = world.read_storage::<Brush>();
    assert_eq!(
        brushes.get(name_map["root"]),
        Some(&Brush::Color(Color::RED))
    );
    assert_eq!(brushes.get(name_map["ent2"]), None);
    assert_eq!(brushes.get(name_map["style1"]), None);
}
