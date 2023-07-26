
use std::collections::HashMap;
use std::string::ToString;
use std::path::PathBuf;
use std::fs;
use std::fmt::{self, Display};

use crate::cli;
use crate::errors::{CliResult,Errors};

pub type Wads = HashMap<String, Wad>;

#[derive(Debug)]
pub enum Error<'a> {
    FilePath(&'a PathBuf),
    FileRead(String),
}

impl<'a>  Display for Error<'a>  {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FilePath(path) => write!(f, "'{}' not found", path.display()),
            Self::FileRead(message) => write!(f, "Could not load wad:`{message}`"),
        }
    }
}




struct Wad {
  path: PathBuf,
  data: Vec<u8>,

}
pub struct Reader {
    pub wads: Wads, 
}

impl Reader {
    pub fn new(args: &cli::Args) -> CliResult<Self> {
        let wads: Wads = 
            args.wad_paths.iter().try_fold(Wads::new(), |mut wads, path| {
                let name = path.file_stem()
                                        .ok_or_else(|| Errors::from(Error::FilePath(path)))?
                                        .to_str().unwrap()
                                        .to_string();

                wads.insert(
                    name,
                    Wad {
                        path: path.clone(),
                        data: fs::read(path).or_else(|err| Err(Errors::from(Error::FileRead(err.to_string()))))?,
                    }
                );
                Ok::<Wads, Errors>(wads)
            })?;
        Ok(Self {
            wads,
        })

    }

}