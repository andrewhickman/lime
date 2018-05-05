use std::cmp::Ordering;
use std::time::Instant;

use ::SECOND;

#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub(crate) enum Action {
    Update = 0,
    Render = 1,
    Log = 2,
}

impl Action {
    pub(crate) const COUNT: usize = 3;

    pub(crate) fn values() -> ActionValues {
        ActionValues { prev: None }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) struct QueuedAction(pub Action, pub Instant);

impl QueuedAction {
    pub(crate) fn new(action: Action) -> Self {
        if action == Action::Log {
            QueuedAction(action, Instant::now() + SECOND)
        } else {
            QueuedAction(action, Instant::now())
        }
    }
}

impl PartialOrd for QueuedAction {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for QueuedAction {
    fn cmp(&self, other: &Self) -> Ordering {
        other.1.cmp(&self.1)
    }
}

pub(crate) struct ActionValues {
    prev: Option<Action>,
}

impl Iterator for ActionValues {
    type Item = Action;

    fn next(&mut self) -> Option<Self::Item> {
        self.prev = match self.prev {
            None => Some(Action::Update),
            Some(Action::Update) => Some(Action::Render),
            Some(Action::Render) => Some(Action::Log),
            Some(Action::Log) => None,
        };

        self.prev
    }
}

#[test]
fn test_action_values() {
    assert_eq!(Action::COUNT, Action::values().count());

    for (idx, action) in Action::values().enumerate() {
        assert_eq!(idx, action as usize);
    }
}