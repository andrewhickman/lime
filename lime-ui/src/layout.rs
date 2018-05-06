use cassowary::Variable;

pub trait Layout {
}

pub struct Position {
    left: Variable,
    right: Variable,
    top: Variable,
    bottom: Variable,
}
