#![allow(unused_imports)]
mod cli;
mod errors;

#[macro_use]
pub mod macros;
mod wad;
mod engine;

use std::collections::HashMap;

use std::time::Duration;


use macros::*;


use engine::GameLoop;

fn main() -> errors::CliResult<'static> {
    let args = cli::args();

    let wad_name = args.wad_paths.first().unwrap().file_stem().unwrap().to_str().unwrap().to_string();
    let engine = engine::Engine::new(&args)?;

    if args.list_maps {
        for map_name_lump in engine.reader.get_map_list(&wad_name).unwrap().iter() {
            println!("{}", map_name_lump.name);
        }
        return Ok(());
    }

    let engine = engine.set_up().unwrap();
    let mut engine = engine.start("DOOM", "E1M1").unwrap();
    
    engine.main_loop();
    Ok(())
}


// fn get_map_bounds(vertex_lumps: &LumpData) -> (i32, i32, i32 ,i32) {


// }