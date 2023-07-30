#![allow(unused_imports)]
mod cli;
mod errors;

mod wad;
mod engine;

use std::collections::HashMap;
const DOOMMAPLUMPLENGTH: usize = 11;

use sdl2::VideoSubsystem;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;


fn main() -> errors::CliResult<'static> {
    let args = cli::args();
    let engine = engine::Engine::new(&args)?;
    let lumps = engine.reader.lumps_for("DOOM2")?;

    let map_data = engine.reader.get_map_lumps("DOOM2", "MAP01")?;

    //println!("{:?}", map_data.line_defs);
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem.window("Example", 1920, 1080).build().unwrap();
    let mut canvas : Canvas<Window> = window.into_canvas()
    .present_vsync() //< this means the screen cannot
    // render faster than your display rate (usually 60Hz or 144Hz)
    .build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.set_draw_color(Color::RGB(255, 210, 0));

    if let wad::LumpData::DeserializeLump(line_Defs) = &map_data.line_defs.data {
        for line_def in line_Defs.iter() {
            if let wad::DeserializeLump::LineDef { start, end, flags, special_type, tag, front, back } = &line_def { 
                if let wad::LumpData::DeserializeLump(vertexes) = &map_data.vertexs.data {
                  let p1 = 
                    if let wad::DeserializeLump::Vertex { x, y }  =  &vertexes[*start as usize] {
                        (*x as i32, *y as i32)
                    } else { println!("LUMP: {:?}", &vertexes[*start as usize]);  panic!()};
                  let p2 = 
                    if let wad::DeserializeLump::Vertex { x, y }  =  &vertexes[*end as usize] {
                        (*x as i32, *y as i32)
                    } else { panic!()};
                  canvas.draw_line(p1, p2).unwrap();
                };
            }
        }
    }
    let mut event_pump = sdl_context.event_pump().unwrap();
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
        canvas.present();
    }
    
  
    Ok(())
}


