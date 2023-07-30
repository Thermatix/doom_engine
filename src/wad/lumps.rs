#![allow(unused_imports)]
use super::*;

use std::{sync::OnceLock, hash::Hash};

use modular_bitfield::prelude::*;
use binrw::{binrw, BinRead, args};


pub static mut LUMPMETA: OnceLock<HashMap<&str, usize>> = OnceLock::new();

pub fn lump_meta() -> &'static mut HashMap<&'static str, usize> {
    unsafe {
        LUMPMETA.get_or_init(|| HashMap::new() );
        LUMPMETA.get_mut().unwrap()
    }
}


#[derive(Debug, BinRead, Clone)]
#[br(little)]
pub struct Lump {
    pub offset: i32,
    pub size: i32,
    #[br(count = 8, map = |x: Vec<u8>| String::from_utf8(x).unwrap())]
    pub name: String,
    #[br(seek_before = SeekFrom::Start(offset as u64),  restore_position)]
    #[br(args {count: Self::lump_size(size, &name), name: &name }, assert(post_process_lump(&name, size, &data)))]
    pub data: LumpData,

}

fn post_process_lump(name: &str, size: i32, lump_data: &LumpData) -> bool {
    let lump_meta = lump_meta();

    if name.starts_with("SECTOR") {
        lump_meta.insert("SECTOR_COUNT", lump_data.count());
    } else if name.starts_with("REJECT") {
        *lump_meta.get_mut("SECTOR_COUNT").unwrap() = 0;
    }

    true
}

fn reject_count() -> usize {
    let lump_meta = lump_meta();
    let sec_count = lump_meta.get("SECTOR_COUNT").unwrap();
    (sec_count * sec_count) / 8

}


impl Lump  {
    fn lump_size(size: i32, name: &str) -> usize {
        if size == 0 { 0 } else {
            (size / if name.starts_with("THING") { 10 }
            else if name.starts_with("LINEDEF") { 14 }
            else if name.starts_with("SIDEDEFS") { 30 }
            else if name.starts_with("VERTEX") { 4 }
            else if name.starts_with("SEG") { 12 }
            else if name.starts_with("SSECTOR") { 4 }
            else if name.starts_with("NODES") { 18 }
            else if name.starts_with("SECTOR") { 26 }
            else if name.starts_with("REJECT") { 1 }
            else if name.starts_with("BLOCKMAP") { 8 }
            else if name.starts_with("BMOFFSET") { 2 }
            else { 1 }) as usize
        }

    }
}


pub fn deserialiazable_lumps(name: &str) -> bool { 
    let whitelist = [
        "THING",
        "LINEDEF",
        "SIDEDEFS",
        "VERTEX",
        "SEG",
        "SSECTOR",
        "NODES",
        "SECTOR",
        "REJECT",
        "BLOCKMAP",
        "BMOFFSET"
    ];
    for prefix in whitelist.iter() {
        if name.starts_with(prefix) {
            return true;
        }
    }
    false  
}


#[derive(Debug, BinRead, Clone, Default)]
#[br(little, import { count: usize, name: &str } )]
pub enum LumpData {

    #[br(pre_assert(deserialiazable_lumps(&name)))] DeserializeLump (
        #[br(args { count: count, inner: args! { name }})]
        Vec<DeserializeLump>
    ),
    #[br(pre_assert(count > 0))] Bytes (
        #[br(args { count: count } )]
        Vec<u8>
    ),
    #[default] Marker

}

impl LumpData {
    pub fn count(&self) -> usize {
        match self {
            Self::DeserializeLump(data) => data.len(),
            Self::Bytes(data) => data.len(),
            Self::Marker => 0
        }
    }
}

#[derive(Debug, BinRead, Clone, Default)]
#[br(little, import { name: &str })]
pub enum DeserializeLump {
    #[br(pre_assert(name.starts_with("THING")))] Thing {
        x: i16,
        y: i16,
        angle_facing: i16,
        doomed_thing_type: i16,
        flags: ThingFlags,
    
    },
    #[br(pre_assert(name.starts_with("LINEDEF")))] LineDef {
        start: i16,
        end: i16,
        flags: LineDefFlags,
        special_type: i16,
        tag: i16,
        front: i16,
        back: i16,
    },

    #[br(pre_assert(name.starts_with("SIDEDEF")))] SideDef {
        x_offset: i16,
        y_offset: i16,
        #[br(count = 8, map = |x: Vec<u8>| String::from_utf8(x).unwrap())]
        name_of_upper: String,
        #[br(count = 8, map = |x: Vec<u8>| String::from_utf8(x).unwrap())]
        name_of_lower: String,
        #[br(count = 8, map = |x: Vec<u8>| String::from_utf8(x).unwrap())]
        name_of_middle: String,
        sector_this_sidedef_faces: i16
    },
    #[br(pre_assert(name.starts_with("VERTEX")))] Vertex {
        x: i16,
        y: i16
    },
    #[br(pre_assert(name.starts_with("SEG")))] Segment {
        start_vertext_id: i16,
        end_verext_id: i16,
        angle: i16, // full circle is -32768 to 32767.
        line_def_id: i16,
        direction: i16, //  0 (same as linedef) or 1 (opposite of linedef)
        offset: i16, // distance along linedef to start of segments 
    },
    #[br(pre_assert(name.starts_with("SSECTOR")))] SubSector {
        segments_count: i16,
        first_segments_id: i16
    },
    #[br(pre_assert(name.starts_with("NODE")))] Node {
        x_partion_line_start: i16,
        y_partion_line_start: i16,
        change_in_x_partion_line_start_to_end: i16,
        change_in_y_partion_line_start_to_end: i16,
        #[br(count = 4)]
        right_bounding_box: Vec<i16>,
        #[br(count = 4)]
        left_bounding_box: Vec<i16>,
        right_child: i16,
        left_child: i16,
    },
    #[br(pre_assert(name.starts_with("SECTOR")))] Sector {
        floor_height: i16,
        ceiling_height: i16,
        #[br(count = 8, map = |x: Vec<u8>| String::from_utf8(x).unwrap())]
        name_of_floor_texture: String,
        #[br(count = 8, map = |x: Vec<u8>| String::from_utf8(x).unwrap())]
        name_of_ceiling_texture: String,
        light_level: i16,
        special_type: i16,
        tag: i16,
    },
    #[br(pre_assert(name.starts_with("REJECT")))] Reject {
        #[br(count = reject_count())]
        table: Vec<u8>
    },
    #[br(pre_assert(name.starts_with("BLOCKMAP")))] BlockMap {
        x_grid_origin: i16,
        y_grid_origin: i16,
        columns: i16,
        rows: i16,
        //#[br(count = (columns as usize * rows as usize))]
        //offsets: Vec<i16>
    },
    #[default] N
}


// #[derive(Debug, Clone, BinRead)]
// #[br(map = |&x| Self::into_bytes(x), little)]
// pub struct RejectTable {
//     #[br(count = reject_count(), map = |r: Vec<u8>| r.chunk(lump_meta().get("SECTOR_COUNT").unwrap()).collect()) ]
//     table: Vec<Vec<bool>>,

// }

fn check_name(name: &str) -> bool {

    println!("{name}");

    true
}

#[bitfield]
#[derive(Debug, Clone)]
#[binrw]
#[br(map = Self::from_bytes, little)]
pub struct ThingFlags {
    pub skill_levels_1_2: bool, 
    pub skill_level_3: bool,
    pub skill_level_4: bool,
    pub deaf: bool,
    pub not_in_singleplayer: bool,
    _unused5: bool,
    _unused6: bool,
    _unused7: bool,
    _unused8: bool,
    _unused9: bool,
    _unused10: bool,
    _unused11: bool,
    _unused12: bool,
    _unused13: bool,
    _unused14: bool,
    _unused15: bool, 
}

#[bitfield]
#[derive(Debug, Clone)]
#[binrw]
#[br(map = Self::from_bytes, little)]
pub struct LineDefFlags {
    pub blocks_player_and_monsters: bool, 
    pub blocks_monsters: bool,
    pub two_sided: bool,
    pub upper_is_unpegged: bool,
    pub lower_is_unpegged: bool,
    pub secret: bool,
    pub blocks_sound: bool,
    pub never_automap: bool,
    pub always_automap: bool,
    _unused9: bool,
    _unused10: bool,
    _unused11: bool,
    _unused12: bool,
    _unused13: bool,
    _unused14: bool,
    _unused15: bool, 
}