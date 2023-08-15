#[allow(unused_imports)]
use super::*;

use std::ops::{DerefMut, Range};
use std::{sync::OnceLock, hash::Hash};
use std::convert::{TryFrom, From};
use std::fmt::{self, Display};

use modular_bitfield::prelude::*;
use binrw::{binrw, args, NamedArgs};
use regex::Regex;
pub use binrw::BinRead;

#[derive(Debug)]
pub enum Error {
    Unwraping(String, String),
    Access(String),
}

impl  Display for Error  {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unwraping(outer, inner) => write!(f, "couldn't unwrap'{outer}' to inner `{inner}`"),
            Self::Access(message) => write!(f, "error accessing LumpData: `{message}`"),
        }
    }
}

pub static mut LUMPMETA: OnceLock<HashMap<&str, usize>> = OnceLock::new();


pub fn lump_meta() -> &'static mut HashMap<&'static str, usize> {
    unsafe {
        LUMPMETA.get_or_init(|| HashMap::new() );
        LUMPMETA.get_mut().unwrap()
    }
}


#[derive(Debug, BinRead)]
#[br(little)]
pub struct Lump {
    pub offset: i32,
    pub size: i32,
    #[br(count = 8, map = |x: Vec<u8>| String::from_utf8(x).unwrap())]
    pub name: String,
    #[br(calc = (&name, size).into())]
    pub kind: LumpKind,
    #[br(calc = Self::lump_count(&kind, size))]
    pub count: usize, 
}

impl Lump {
    pub fn deserialize<T:for<'a> BinRead<Args<'a>=()> + 'static>(&self, raw_data: &super::RawData) -> Vec<T> {
        let mut cursor = Cursor::new(raw_data);
        cursor.seek(SeekFrom::Start(self.offset as u64)).unwrap();
        
        let result = <Vec<T> as BinRead>::read_le_args(&mut cursor, args! { count: self.count }).unwrap();
        self.post_process();
        result
    }

    fn lump_count(kind: &LumpKind, size: i32) -> usize {
        if size == 0 { 0 } else {
            size as usize / (match kind {
                LumpKind::Things => 10,
                LumpKind::LineDefs => 14,
                LumpKind::SideDefs => 30,
                LumpKind::Vertexs => 4,
                LumpKind::Segments => 12,
                LumpKind::SubSectors => 4,
                LumpKind::Nodes => 28,
                LumpKind::Sectors => 26,
                LumpKind::Rejects => 1,
                LumpKind::BlockMaps => 8,
                LumpKind::BlockMapOffset=> 2,
                _ => 1,
            } as usize)
        }
    }

    fn post_process(&self) {
        match self.kind {
            LumpKind::Sectors => lump_meta().insert("SECTOR_COUNT", self.count),
            LumpKind::Rejects => lump_meta().insert("SECTOR_COUNT", 0),
            _ => None, 
        };
        lump_meta().insert("ID", 0);
    }
}

fn get_and_incriment_id() -> u16 {
    let id = lump_meta().entry("ID").or_insert(0); 
    lump_meta().insert("ID", *id + 1);
    *id as u16
}

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum LumpKind {
    Pallet,
    ColourMap,
    AnsiText,
    Demo,
    MapMarker,
    Things,
    LineDefs,
    SideDefs,
    Vertexs,
    Segments,
    SubSectors,
    Nodes,
    Sectors,
    Rejects,
    BlockMaps,
    BlockMapOffset,
    DataMArker,
    Texture,
    PNames,
    GenMidi,
    Dmxgus,
    SoundSpeaker,
    SoundDoomFormat,
    Music,
    Graphics,
    StartMarker,
    EndMarker,
}

impl From<(&String, i32)> for LumpKind {
    fn from((name, size): (&String, i32)) -> Self {
        let lump_kind_matcher = Regex::new(
            r#"(?x)
            (PLAYPAL) |
            (COLORMAP) |
            (ENDOOM)|
            (DEMO)|
            (THINGS)|
            (LINEDEFS)|
            (SIDEDEFS)|
            (VERTEXES)|
            (SEGS)|
            (SSECTORS)|
            (NODES)|
            (SECTORS)|
            (REJECT)|
            (BLOCKMAP)|
            (TEXTURE)|
            (PNAMES)|
            (GENMIDI)|
            (DMXGUS)|
            (DP)|
            (DS)|
            (D_)|
            (_START)|
            (_END)\s+(\d+)
            "#).unwrap();
        let captures = lump_kind_matcher.captures(name).map(|captures| {
            captures
                .iter() // All the captured groups
                .skip(1) // Skipping the complete match
                .flat_map(|c| c) // Ignoring all empty optional matches
                .map(|c| c.as_str()) // Grab the original strings
                .collect::<Vec<_>>() // Create a vector
        });
    
        match captures.as_ref().map(|c| c.as_slice()) {
            Some(["PLAYPAL"]) => Self::Pallet,
            Some(["COLORMAP"]) => Self::ColourMap,
            Some(["ENDOOM"]) => Self::AnsiText,
            Some(["DEMO"]) => Self::Demo,
            Some(["THINGS"]) => Self::Things,
            Some(["LINEDEFS"]) => Self::LineDefs,
            Some(["SIDEDEFS"]) => Self::SideDefs,
            Some(["VERTEXES"]) => Self::Vertexs,
            Some(["SEGS"]) => Self::Segments,
            Some(["SSECTORS"]) => Self::SubSectors,
            Some(["NODES"]) => Self::Nodes,
            Some(["SECTORS"]) => Self::Sectors,
            Some(["REJECT"]) => Self::Rejects,
            Some(["BLOCKMAP"]) => Self::BlockMaps,
            Some(["TEXTURE"]) => Self::Texture,
            Some(["PNAMES"]) => Self::PNames,
            Some(["GENMIDI"]) => Self::GenMidi,
            Some(["DMXGUS"]) => Self::Dmxgus,
            Some(["DP"]) => Self::SoundSpeaker,
            Some(["DS"]) => Self::SoundDoomFormat,
            Some(["D_"]) => Self::Music,
            Some(["_START"]) => Self::StartMarker,
            Some(["_END"]) => Self::EndMarker,
            _ => if size > 0 { Self::Graphics } else { Self::MapMarker }
        }
    }
}

fn reject_count() -> usize {
    let lump_meta = lump_meta();
    let sec_count = lump_meta.get("SECTOR_COUNT").unwrap();
    (sec_count * sec_count) / 8
}

fn bytes_to_string(bytes: Vec<u8>) -> String {
    bytes.iter().map(|b| char::from(*b)).collect()
}


#[derive(Debug, BinRead, PartialEq, Eq, Copy, Clone)]
#[br(little)]
pub struct Thing {
    pub x: i16,
    pub y: i16,
    pub angle_facing: i16,
    pub doomed_thing_type: i16,
    pub flags: ThingFlags,
    #[br(calc = get_and_incriment_id())]
    pub id: u16,
}

#[derive(Debug, BinRead, PartialEq, Eq, Copy, Clone)]
#[br(little)]
pub struct LineDef {
    pub start_vertex_id: u16,
    pub end_vertex_id: u16,
    pub flags: LineDefFlags,
    pub special_type: i16,
    pub tag: i16,
    pub front: i16,
    pub back: i16,
    #[br(calc = get_and_incriment_id())]
    pub id: u16,
}

#[derive(Debug, BinRead, PartialEq, Eq)]
#[br(little)]
pub struct SideDef {
    pub x_offset: i16,
    pub y_offset: i16,
    #[br(count = 8, map = |x: Vec<u8>| bytes_to_string(x))]
    pub name_of_upper: String,
    #[br(count = 8, map = |x: Vec<u8>| bytes_to_string(x))]
    pub name_of_lower: String,
    #[br(count = 8, map = |x: Vec<u8>| bytes_to_string(x))]
    pub name_of_middle: String,
    pub sector_this_sidedef_faces: i16,
    #[br(calc = get_and_incriment_id())]
    pub id: u16,
}

#[derive(Debug, BinRead, PartialEq, Eq, Copy, Clone)]
#[br(little)]
pub struct Vertex {
    pub x: i16,
    pub y: i16,
    #[br(calc = get_and_incriment_id())]
    pub id: u16,
}

#[derive(Debug, BinRead, PartialEq, Eq, Copy, Clone)]
#[br(little)]
pub struct Segment {
    pub start_vertext_id: u16,
    pub end_verext_id: u16,
    pub angle: i16, // full circle is -32768 to 32767.
    pub line_def_id: u16,
    pub direction: SegDirection,
    pub offset: i16, // distance along linedef to start of segments 
    #[br(calc = get_and_incriment_id())]
    pub id: u16,
}

#[derive(Debug, BinRead, PartialEq, Eq, Copy, Clone)]
#[br(little)]
pub struct SubSector {
    pub segments_count: u16,
    pub first_segments_id: u16,
    #[br(calc = get_and_incriment_id())]
    pub id: u16,
}

impl SubSector {
    pub const SUB_SECTOR_IDENTIFIER: u16 = 0x8000;
    pub const IDENTIFIER_BITMASK: u16 = 0b0111111111111111;

    pub fn to_range(&self) -> core::ops::Range<usize> {
        (self.first_segments_id as usize)..((self.first_segments_id + self.segments_count) as usize)
    }
}

#[derive(Debug, BinRead, PartialEq, Eq, Copy, Clone)]
#[br(little)]
pub struct Node {
    pub x_partion: i16,
    pub y_partion: i16,
    pub dx_partion: i16,
    pub dy_partion: i16, 
    pub front_bbox: BoundingBox,
    pub back_bbox: BoundingBox,
    pub front_child_id: u16,
    pub back_child_id: u16,
    #[br(calc = get_and_incriment_id())]
    pub id: u16,
}

impl Node {

    pub fn children_are_sub_sectors(&self) -> (bool, bool) {
        (
            self.back_child_id  >= SubSector::SUB_SECTOR_IDENTIFIER,
            self.front_child_id >= SubSector::SUB_SECTOR_IDENTIFIER,
        )
    }

    pub fn is_in_back_side(&self, (player_x, player_y): (i16, i16)) -> bool {
        let dx = (player_x - self.x_partion) as i32;
        let dy = (player_y - self.y_partion) as i32;
        dx * self.dy_partion as i32 - dy * self.dx_partion as i32 <= 0
    }

}

#[derive(Debug, BinRead, PartialEq, Eq)]
#[br(little)]
pub struct Sector {
    pub floor_height: i16,
    pub ceiling_height: i16,
    #[br(count = 8, map = |x: Vec<u8>| String::from_utf8(x).unwrap())]
    pub name_of_floor_texture: String,
    #[br(count = 8, map = |x: Vec<u8>| String::from_utf8(x).unwrap())]
    pub name_of_ceiling_texture: String,
    pub light_level: i16,
    pub special_type: i16,
    pub tag: i16,
    #[br(calc = get_and_incriment_id())]
    pub id: u16,
}
#[derive(Debug, BinRead, PartialEq, Eq, Clone)]
#[br(little)]
pub struct Reject {
    #[br(count = Self::reject_count())]
    pub table: Vec<u8>,
    #[br(calc = get_and_incriment_id())]
    pub id: u16,
}

impl Reject {
    pub fn reject_count() -> usize {
        let lump_meta = lump_meta();
        let sec_count = lump_meta.get("SECTOR_COUNT").unwrap();
        (sec_count * sec_count) / 8
    }
}

#[derive(Debug, BinRead, PartialEq, Eq)]
#[br(little)]
pub struct BlockMap {
    pub x_grid_origin: i16,
    pub y_grid_origin: i16,
    pub columns: i16,
    pub rows: i16,
    #[br(calc = get_and_incriment_id())]
    pub id: u16,
    //#[br(count = (columns as usize * rows as usize))]
    //pub offsets: Vec<i16>
}

#[derive(Debug, BinRead, PartialEq, Eq, Copy, Clone)]
#[br(little,  repr = i16)]
pub enum SegDirection {
    SameAsLineDef = 0,
    OppositOfLineDef = 1,
}

// top, bottom, left and righ
#[derive(Debug, BinRead, PartialEq, Eq, Copy, Clone)]
pub struct BoundingBox {
    pub h: i16,
    pub y: i16,
    pub w: i16,
    pub x: i16,
}

#[bitfield]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
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
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
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