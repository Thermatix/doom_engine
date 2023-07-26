use std::fmt::{self, Display};
use std::convert::From;


use crate::cli;
use crate::wad;
use crate::engine;

pub type CliResult<'a,Good=()> = Result<Good,Errors<'a>>;

#[derive(Debug)]
pub enum Errors<'a>{
    Wad(wad::Error<'a>),
    Engine(engine::Error<'a>)
}


impl<'a>  Display for Errors<'a>  {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Wad(error) => write!(f,"Wad error: {error}"),
            Self::Engine(error) => write!(f,"Engine error: {error}"),
        }
    }
}

impl<'a> From<wad::Error<'a>> for Errors<'a> {
    fn from(error: wad::Error<'a>) -> Self {
        Self::Wad(error)
    }
}

impl<'a> From<engine::Error<'a>> for Errors<'a> {
    fn from(error: engine::Error<'a>) -> Self {
        Self::Engine(error)
    }
}
