#![allow(unused_imports)]

mod player;
mod draw;
mod errors;

use std::marker::PhantomData;
use std::{thread, time};

use crate::cli;
use crate::errors::{CliResult,Errors};

use crate::wad;


use draw::Manager;


use sdl2::{
    VideoSubsystem,
    render::Canvas,
    video::Window,
    pixels::Color,
    event::Event,
    keyboard::Keycode,
};


pub use errors::*;

use self::player::Player;

// Game State Markers

pub struct Init {}
pub struct MainMenu {}
pub struct InGameMenu {}
pub struct InGame {}


//#[derive(Debug)]
pub struct Engine<State, Draw = draw::Draw2D> 
    where Draw: Manager {
    pub reader: wad::Reader,
    pub context: Option<Context>,
    pub sdl_context: sdl2::Sdl,
    video: VideoSubsystem,
    //window: Window,
    canvas: Canvas<Window>,
    pub draw: Draw,
    _state: PhantomData<State>,
}

#[derive(Debug)]
pub struct Context {
    pub current_map: wad::Map,
    pub player: Player,
}


impl Engine<Init> {
    pub fn new(args: &cli::Args) -> CliResult<Self> {
        let reader = wad::Reader::new(args)?;
        let sdl_context = sdl2::init().unwrap();
        let draw = draw::Draw2D::new(args.screen_width, args.screen_height);
        let video = sdl_context.video().unwrap();
        let window = video.window("Map Display", args.screen_width as u32, args.screen_height as u32)
                                .build().unwrap();
        let canvas : Canvas<Window> = window.into_canvas()
                                                .present_vsync() //<this means the screen cannot
                                                // render faster than your display rate (usually 60Hz or 144Hz)
                                                .build().unwrap();
        Ok(Self {
            reader,
            context: None,
            sdl_context,
            video,
            canvas,
            draw,
            _state: PhantomData::default(),

        })

    }
}

impl Engine<Init> {
    pub fn set_up(self) -> CliResult<'static, Engine<MainMenu>> {
        Ok(Engine {
            reader: self.reader,
            context: self.context,
            sdl_context: self.sdl_context,
            video: self.video,
            canvas: self.canvas,
            draw: self.draw,
            _state: PhantomData::default(),
        })
    }
}
impl Engine<MainMenu> {
    pub fn start(self, wad_name: &str, map_name: &str) -> CliResult<'static, Engine<InGame>> {
        let map = self.reader.get_map(wad_name, map_name).unwrap();
        let player_thing: wad::Thing = map.things[0].clone().into();

        let player = Player::new(player_thing);

        let context = Context {
            current_map: map,
            player: player,
        };

        Ok(Engine {
            reader: self.reader,
            context: Some(context),
            sdl_context: self.sdl_context,
            video: self.video,
            canvas: self.canvas,
            draw: self.draw,
            _state: PhantomData::default(),
        })
    }
}

impl Engine<InGame> {
    pub fn pause(self) -> CliResult<'static, Engine<InGameMenu>> {
        // use std::{thread, time};
        // let one_second = time::Duration::from_secs(1);
        // thread::sleep(one_second);
        Ok(Engine {
            reader: self.reader,
            context: self.context,
            sdl_context: self.sdl_context,
            video: self.video,
            canvas: self.canvas,
            draw: self.draw,
            _state: PhantomData::default(),
        })
    }
}

impl GameLoopStages for Engine<MainMenu> {

    fn update(&mut self) {
        
    }

    fn render(&mut self) {  
        
    }
}

impl GameLoopStages for Engine<InGame> {

    fn input(&self, event: Event) -> Option<Action> {
        match event {
            Event::Quit {..} |
            Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                Some(Action::Quit)
            },
            _ => {None}
        }
    }

    fn update(&mut self) {
        
    }

    fn render(&mut self) {
        self.draw.draw_layers(&mut self.canvas, self.context.as_mut().unwrap());
    }
}

impl GameLoopStages for Engine<InGameMenu>  {

    fn update(&mut self) {
        
    }

    fn render(&mut self) {  
        
    }
}


/// Blanket implimentation for any types that impliment `GameLoopStages`,
/// Provides Main process loop and events (and possibly other common processes)
impl<State> GameLoop for Engine<State> where Engine<State>: GameLoopStages {

    fn main_loop(&mut self) {
        let mut event_pump = self.sdl_context.event_pump().unwrap();
        let ten_millis = time::Duration::from_millis(10);
        'running: loop {
            for event in event_pump.poll_iter() {
                if let Some(action) = self.input(event) {
                    match action {
                        Action::Quit => break 'running
                    }
                }
                self.update();
                self.render();
                self.canvas.present(); 
                thread::sleep(ten_millis);
            }
        }
    }  
}

pub enum Action {
    Quit
}

pub trait GameLoopStages {
    fn input(&self, _event: Event) -> Option<Action> {
        None
    }

    fn update(&mut self);

    fn render(&mut self);
}
pub trait GameLoop: GameLoopStages {

    fn main_loop(&mut self) {
        
    }
}