use std::fmt::{self, Display};
use std::convert::From;


use crate::cli;
use crate::wad;
use crate::engine;

pub type CliResult<'a,Good=()> = Result<Good,Errors>;

#[derive(Debug)]
pub enum Errors{
    Wad(wad::Error),
    Engine(engine::Error)
}


impl  Display for Errors  {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Wad(error) => write!(f,"Wad error: {error}"),
            Self::Engine(error) => write!(f,"Engine error: {error}"),
        }
    }
}

impl From<wad::Error> for Errors {
    fn from(error: wad::Error) -> Self {
        Self::Wad(error)
    }
}

impl From<engine::Error> for Errors {
    fn from(error: engine::Error) -> Self {
        Self::Engine(error)
    }
}
