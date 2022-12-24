use std::fmt::Display;

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum Team {
    Neutral,
    Red,
    Blue,
}

impl Display for Team {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Team::Blue => write!(f, "Blue"),
            Team::Red => write!(f, "Red"),
            Team::Neutral => write!(f, "Neutral"),
        }
    }
}
