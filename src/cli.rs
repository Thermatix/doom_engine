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
    pub wad_paths: Vec<PathBuf>,
    #[arg(short, long,  value_name = "map_name")]
    pub map_name: Option<String>,
    #[arg(short = 'l', long)]
    pub list_maps: bool,
    #[arg(short = 'x', long, default_value = "800")]
    pub screen_width: i32,
    #[arg(short = 'y', long, default_value = "600")]
    pub screen_height: i32,

}