use super::*;

#[test]
fn flags() {
    assert_eq!(
        StateFlags::NEEDS_LAYOUT.dependencies(),
        StateFlags::NEEDS_LAYOUT
    );
    assert_eq!(
        StateFlags::NEEDS_LAYOUT.dependants(),
        StateFlags::NEEDS_LAYOUT | StateFlags::NEEDS_DRAW | StateFlags::NEEDS_EVENTS
    );

    assert_eq!(
        StateFlags::NEEDS_DRAW.dependencies(),
        StateFlags::NEEDS_LAYOUT | StateFlags::NEEDS_DRAW
    );
    assert_eq!(
        StateFlags::NEEDS_DRAW.dependants(),
        StateFlags::NEEDS_DRAW | StateFlags::NEEDS_EVENTS
    );

    assert_eq!(
        StateFlags::NEEDS_EVENTS.dependencies(),
        StateFlags::NEEDS_LAYOUT | StateFlags::NEEDS_DRAW | StateFlags::NEEDS_EVENTS
    );
    assert_eq!(
        StateFlags::NEEDS_EVENTS.dependants(),
        StateFlags::NEEDS_EVENTS
    );
}
