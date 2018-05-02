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