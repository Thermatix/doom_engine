#![allow(unused_imports)]

mod bsp;
mod player;
mod draw;
mod errors;

use std::marker::PhantomData;

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
    pub context: Context,
    pub sdl_context: sdl2::Sdl,
    video: VideoSubsystem,
    //window: Window,
    canvas: Canvas<Window>,
    pub draw: Draw,
    _state: PhantomData<State>,
}

#[derive(Debug)]
pub struct Context {
    pub current_map: Option<wad::Map>,
    pub player: Option<Player>,
}

impl Context {

    pub fn new() -> Self {
        Self {
            current_map: None,
            player: None,
        }
    }
}

impl Engine<Init> {
    pub fn new(args: &cli::Args) -> CliResult<Self> {
        let reader = wad::Reader::new(args)?;
        let sdl_context = sdl2::init().unwrap();
        let draw = draw::Draw2D::new(args.screen_width, args.screen_height);
        let video = sdl_context.video().unwrap();
        let window = video.window("Map Display", args.screen_width as u32, args.screen_height as u32)
                                .build().unwrap();
        let mut canvas : Canvas<Window> = window.into_canvas()
                                                .present_vsync() //<this means the screen cannot
                                                // render faster than your display rate (usually 60Hz or 144Hz)
                                                .build().unwrap();
        Ok(Self {
            reader,
            context: Context::new(),
            sdl_context,
            video,
            canvas,
            draw,
            _state: PhantomData::default(),

        })

    }
}

impl<'e> Engine<Init> {
    pub fn set_up(mut self) -> CliResult<'e,Engine<MainMenu>> {
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
impl<'e> Engine<MainMenu> {
    pub fn start(mut self, wad_name: &str, map_name: &str) -> CliResult<'e, Engine<InGame>> {
        let map = self.reader.get_map(wad_name, map_name).unwrap();
        let player_thing: wad::Thing = map.things.lump_data_deserialized()[0].clone().into();

        let player = Player::new(player_thing);
        self.context.current_map = Some(map);
        self.context.player = Some(player);
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

impl<'e> Engine<InGame> {
    pub fn pause(mut self) -> CliResult<'e, Engine<InGameMenu>> {
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
        self.draw.draw_layers(&mut self.canvas, &mut self.context);
    }
}

impl GameLoopStages for Engine<InGameMenu>  {

    fn update(&mut self) {
        
    }

    fn render(&mut self) {  
        
    }
}

impl<State> GameLoop for Engine<State> where Engine<State>: GameLoopStages {

    fn main_loop(&mut self) {
        let mut event_pump = self.sdl_context.event_pump().unwrap();
        'running: loop {
            for event in event_pump.poll_iter() {
                if let Some(action) = self.input(event) {
                    match action {
                        Action::Quit => break 'running
                    }
                }
                self.update();
                self.render();
                &self.canvas.present(); 
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