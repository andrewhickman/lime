use render::Color;
use serde_json as json;
use specs::prelude::*;

use super::*;
use de;
use draw::Brush;
use widget::button::{ButtonStyle, ToggleButtonStyle};

#[test]
fn de() {
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
        }
    }
    "##;

    let mut world = World::new();
    let registry = de::Registry::new();
    world.register::<Brush>();
    world.register::<Style>();
    world.register::<ButtonStyle>();
    world.register::<ToggleButtonStyle>();

    de::deserialize(
        &mut json::Deserializer::from_str(DATA),
        &registry,
        &mut world.res,
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
