use std::fmt::{self, Display};

use crate::cli;
use crate::errors::{CliResult,Errors};

use crate::wad;

#[derive(Debug)]
pub enum Error<'a> {
    NoErrors(&'a str)
}

impl<'a>  Display for Error<'a>  {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoErrors(_) => write!(f, "No Errors not found"),
        }
    }
}



pub struct Engine {
    
    wads: wad::Reader,
}

impl Engine {
    pub fn new(args: &cli::Args) -> CliResult<Self> {
        let wads = wad::Reader::new(args)?;
        Ok(Self {
            wads
        })

    }
}