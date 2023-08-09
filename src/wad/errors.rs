use std::path::PathBuf;
use std::fmt::{self, Display};

pub use crate::errors::*;
use super::lumps;

#[derive(Debug)]
pub enum Error {
    FilePath(PathBuf),
    FileOpen(String),
    FileRead(String),
    Unpacking(UnpackError),
    Reader(String),
    Lump(lumps::Error)
}

impl  Display for Error  {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FilePath(path) => write!(f, "'{}' not found", path.display()),
            Self::FileOpen(message) => write!(f, "Could not Read wad:`{message}`"),
            Self::FileRead(message) => write!(f, "Could not Open wad:`{message}`"),
            Self::Unpacking(unpack_error) => write!(f, "{unpack_error}"),
            Self::Reader(message) => write!(f, "Wad Reader Error: `{message}`"),
            Self::Lump(lumps_error) => write!(f, "Lump processing error: `{lumps_error}`"),
        }
    }
}

impl From<lumps::Error> for Error {
    fn from(lumps_error: lumps::Error) -> Self {
        Self::Lump(lumps_error)
    }
}

impl From<UnpackError> for Error {
    fn from(unpack_error: UnpackError) -> Self {
        Self::Unpacking(unpack_error)
    }
}

#[derive(Debug)]
pub enum UnpackError {
    Headers(String),
}

impl  Display for UnpackError  {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Headers(reason) => write!(f, "Failed to unpack Headers: '{}'", reason.to_string()),
        }
    }
}
