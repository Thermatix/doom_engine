#![allow(unused_imports)]
mod cli;
mod errors;

mod wad;
mod engine;

use std::collections::HashMap;

use std::time::Duration;



use sdl2::{
    VideoSubsystem,
    render::Canvas,
    video::Window,
    pixels::Color,
    event::Event,
    keyboard::Keycode,
};

use wad::LumpData;


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

    let map_name = args.map_name.as_ref().unwrap();
    //let map_data = engine.reader.get_map_lumps("DOOM2", "MAP01")?;
    let map_data = engine.reader.get_map_lumps(&wad_name, &map_name)?;

    for sd in map_data.side_defs.lump_data_deserialized().iter() {
        println!("{sd:?}");
    }

    let width = 800;
    let height = 600;

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem.window("Map Display", width, height).build().unwrap();
    let mut canvas : Canvas<Window> = window.into_canvas()
                                            .present_vsync() //< this means the screen cannot
                                            // render faster than your display rate (usually 60Hz or 144Hz)
                                            .build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    let points = map_data.scale_map_points(width as i32, height as i32, 30);

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut draw_map = true;
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }

        if draw_map {
            for (p1, p2) in map_data.line_defs_to_vertexes(Some(&points))?  {
                canvas.set_draw_color(Color::GREY);
                canvas.draw_line(*p1, *p2).unwrap();
                canvas.set_draw_color(Color::YELLOW);
                canvas.draw_point(*p1).unwrap();
                canvas.draw_point(*p2).unwrap();
                canvas.present();   
                ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
            }
            draw_map = false;
        }
        
    }
    
  
    Ok(())
}


// fn get_map_bounds(vertex_lumps: &LumpData) -> (i32, i32, i32 ,i32) {


// }