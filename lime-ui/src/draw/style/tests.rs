use render::Color;
use serde_json as json;
use specs::prelude::*;

use super::*;
use de;
use draw::Brush;
use widget::button::{ButtonStyle, ToggleButtonStyle};

#[test]
fn de() {
    const DATA: &'static str = r#"
    {
        "style1": {
            "ButtonStyle": {
                "disabled": { "Color": {
                    "r": 0.2, "g": 0.2, "b": 0.2, "a": 1.0
                } },
                "normal": { "Color": {
                    "r": 1.0, "g": 0.0, "b": 0.0, "a": 1.0
                } },
                "focused": { "Color": {
                    "r": 0.0, "g": 1.0, "b": 0.0, "a": 1.0
                } },
                "pressed": { "Color": {
                    "r": 0.0, "g": 0.0, "b": 1.0, "a": 1.0
                } }
            },
            "ToggleButtonStyle": {
                "disabled_on": { "Color": {
                    "r": 0.2, "g": 0.2, "b": 0.2, "a": 1.0
                } },
                "normal_on": { "Color": {
                    "r": 1.0, "g": 0.0, "b": 0.0, "a": 1.0
                } },
                "focused_on": { "Color": {
                    "r": 0.0, "g": 1.0, "b": 0.0, "a": 1.0
                } },
                "pressed_on": { "Color": {
                    "r": 0.0, "g": 0.0, "b": 1.0, "a": 1.0
                } },
                "disabled_off": { "Color": {
                    "r": 0.2, "g": 0.2, "b": 0.2, "a": 1.0
                } },
                "normal_off": { "Color": {
                    "r": 1.0, "g": 0.0, "b": 0.0, "a": 1.0
                } },
                "focused_off": { "Color": {
                    "r": 0.0, "g": 1.0, "b": 0.0, "a": 1.0
                } },
                "pressed_off": { "Color": {
                    "r": 0.0, "g": 0.0, "b": 1.0, "a": 1.0
                } }
            }
        },
        "ent1": {
            "Brush": {
                "Color": {
                    "r": 1.0,
                    "g": 0.0,
                    "b": 0.0,
                    "a": 1.0
                }
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
        }
    }
    "#;

    let mut world = World::new();
    let registry = de::Registry::new();
    world.register::<Brush>();
    world.register::<Style>();
    world.register::<ButtonStyle>();
    world.register::<ToggleButtonStyle>();

    de::deserialize(
        &mut json::Deserializer::from_str(DATA),
        &registry,
        &world.res,
    ).unwrap();
    world.maintain();

    let ents: Vec<Entity> = (&*world.entities()).join().collect();
    assert_eq!(ents.len(), 3);

    let styles = world.read_storage::<Style>();
    assert!(styles.get(ents[0]).is_none());
    assert_eq!(styles.get(ents[1]).unwrap().get(), ents[0]);
    assert!(styles.get(ents[1]).unwrap().is::<ButtonStyle>());
    assert_eq!(styles.get(ents[2]).unwrap().get(), ents[0]);
    assert!(styles.get(ents[2]).unwrap().is::<ToggleButtonStyle>());

    let brushes = world.read_storage::<Brush>();
    assert_eq!(brushes.get(ents[0]), None);
    assert_eq!(brushes.get(ents[1]), Some(&Brush::Color(Color::RED)));
    assert_eq!(brushes.get(ents[2]), None);
}
