#![allow(unused_imports)]
use std::fmt::{self, Display};

use crate::cli;
use crate::errors::{CliResult,Errors};

use crate::wad;

use binrw::{
    binrw,    // #[binrw] attribute
    BinRead,  // trait for reading
    BinWrite, // trait for writing
    BinReaderExt, 
    io::Cursor,
};

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


#[derive(Debug)]
pub struct Engine {
    pub reader: wad::Reader,
}

impl Engine {
    pub fn new(args: &cli::Args) -> CliResult<Self> {
        let reader = wad::Reader::new(args)?;
        Ok(Self {
            reader
        })

    }
}