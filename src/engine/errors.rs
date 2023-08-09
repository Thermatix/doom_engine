use std::fmt::{self, Display};

#[derive(Debug)]
pub enum Error {
    PlayerThing(String)
}

impl  Display for Error  {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PlayerThing(m) => write!(f, "Engine error, couldn't get player's initial data: `{m}`"),
        }
    }
}