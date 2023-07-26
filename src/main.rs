

mod cli;
mod errors;

mod wad;
mod engine;



fn main() -> errors::CliResult<'static> {
    let args = cli::args();
    let wads = wad::Reader::new(&args);
    let doom = engine::Engine::new(&args);
    Ok(())
}

