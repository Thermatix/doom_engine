use std::path::PathBuf;
use std::sync::OnceLock;

use clap::Parser;

pub static ARGS: OnceLock<Args> = OnceLock::new();

pub fn args() -> &'static Args {
    ARGS.get_or_init(|| Args::parse())
}

pub type Paths = Vec<PathBuf>;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// wad data paths
    #[arg(value_delimiter = ',', value_name = "WAD")]
    pub wad_paths: Vec<PathBuf>

}