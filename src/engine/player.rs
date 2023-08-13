use super::*;

use wad::{ThingFlags, Thing};


#[derive(Debug)]
pub struct Player {
    pub x: i16,
    pub y: i16,
    pub angle: i16,
    pub doomed_thing_type: i16,
    pub flags: ThingFlags,
}

impl Player {

    pub fn new(player_thing: Thing) -> Self {
        Self {
            x: player_thing.x,
            y: player_thing.y,
            angle: player_thing.angle_facing,
            doomed_thing_type: player_thing.doomed_thing_type,
            flags: player_thing.flags,
        }
    }

    pub fn update_dir(&mut self, x: i16, y: i16, angle: i16) {
        self.x = x;
        self.y = y;
        self.angle = angle;
    }
}