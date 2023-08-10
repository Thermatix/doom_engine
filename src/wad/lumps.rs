#[allow(unused_imports)]
use super::*;

use std::ops::{DerefMut, Range};
use std::{sync::OnceLock, hash::Hash};
use std::convert::{TryFrom, From};
use std::fmt::{self, Display};

use modular_bitfield::prelude::*;
use binrw::{binrw, BinRead, args};

pub type DeserializedLumps = Vec<DeserializeLump>;

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


#[derive(Debug, BinRead, Clone)]
#[br(little)]
pub struct Lump {
    pub offset: i32,
    pub size: i32,
    #[br(count = 8, map = |x: Vec<u8>| String::from_utf8(x).unwrap())]
    pub name: String,
    #[br(calc = Self::lump_count(size, &name))]
    pub count: usize, 
    #[br(seek_before = SeekFrom::Start(offset as u64),  restore_position)]
    #[br(args {count: count, name: &name }, assert(Self::post_process_lump(&name, &count, &data)))]
    pub data: LumpData,

}

impl Lump {
    pub fn lump_data_bytes(&self) -> &Vec<u8>  {
        match &self.data {
            LumpData::Bytes(bytes) => bytes,
            _ => panic!("called lump_bytes when data is deserialized or a marker: {}", self.data.name_to_string())
        }
    }
    

    pub fn lump_data_deserialized(&self) -> &DeserializedLumps  {
        match &self.data {
            LumpData::DeserializeLump(data) => &data,
            _ => panic!("called lump_deserialized when data is bytes or a marker: {}", self.data.name_to_string())
        }
    }

    fn lump_count(size: i32, name: &str) -> usize {
        if size == 0 { 0 } else {
            (size / if name.starts_with("THING") { 10 }
            else if name.starts_with("LINEDEF") { 14 }
            else if name.starts_with("SIDEDEFS") { 30 }
            else if name.starts_with("VERTEX") { 4 }
            else if name.starts_with("SEG") { 12 }
            else if name.starts_with("SSECTOR") { 4 }
            else if name.starts_with("NODES") { 28 }
            else if name.starts_with("SECTOR") { 26 }
            else if name.starts_with("REJECT") { 1 }
            else if name.starts_with("BLOCKMAP") { 8 }
            else if name.starts_with("BMOFFSET") { 2 }
            else { 1 }) as usize
        }

    }


    fn post_process_lump(name: &String, count: &usize, lump_data: &LumpData) -> bool {
        let lump_meta = lump_meta();

        if name.starts_with("SECTOR") {
            lump_meta.insert("SECTOR_COUNT", *count);
        } else if name.starts_with("REJECT") {
            lump_meta.insert("SECTOR_COUNT", 0);
        }
        true
    }
}



fn reject_count() -> usize {
    let lump_meta = lump_meta();
    let sec_count = lump_meta.get("SECTOR_COUNT").unwrap();
    (sec_count * sec_count) / 8

}





#[derive(Debug, BinRead, Clone, Default, PartialEq, Eq)]
#[br(little, import { count: usize, name: &str } )]
pub enum LumpData {

    #[br(pre_assert(Self::deserialiazable_lumps(&name)))] DeserializeLump (
        #[br(args { count: count, inner: args! { name }})]
        DeserializedLumps
    ),
    #[br(pre_assert(count > 0))] Bytes (
        #[br(args { count: count } )]
        Vec<u8>
    ),
    #[br(pre_assert(!name.contains("_END") || !name.contains("_START")))] MapName,
    #[default] Marker

}




impl<'a> TryFrom<&'a LumpData> for &'a DeserializedLumps {
    type Error = Error;

    fn try_from(ld: &'a LumpData) -> Result<Self, Self::Error> {
        match ld {
            LumpData::DeserializeLump(d) => Ok(&d),
            _ => Err(Self::Error::Unwraping("LumpData".to_string(), "DeserializeLump".to_string())),
        }
    }

}

impl<'a> TryFrom<&'a LumpData> for &'a Vec<u8> {
    type Error = Error;

    fn try_from(ld: &'a LumpData) -> Result<Self, Self::Error> {
        match ld {
            LumpData::Bytes(d) => Ok(&d),
            _ => Err(Self::Error::Unwraping("LumpData".to_string(), "Bytes".to_string())),
        }
    }
}

impl LumpData {
    pub fn name_to_string(&self) -> String {
        match self {
            Self::DeserializeLump(_) => "DeserializedLump".to_string(),
            Self::Bytes(_) => "Bytes".to_string(),
            Self::Marker => "Marker".to_string(),
            Self::MapName => "MapNname".to_string(),
        }

    }

    pub fn count(&self) -> usize {
        match self {
            Self::DeserializeLump(data) => data.len(),
            Self::Bytes(data) => data.len(),
            Self::Marker => 0,
            Self::MapName => 0,
        }
    }



    fn deserialiazable_lumps(name: &str) -> bool { 
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
}

// ERROR invalid utf-8 sequence of 1 bytes from index 4, length of: 1 for : [45, 0, 0, 0, 253, 4, 0, 0]

fn bytes_to_string(bytes: Vec<u8>) -> String {
    bytes.iter().map(|b| char::from(*b)).collect()
}


#[derive(Debug, BinRead, Clone, Default, PartialEq, Eq)]
#[br(little, import { name: &str })]
pub enum DeserializeLump {
    #[br(pre_assert(name.starts_with("THING")))]  Thing(Thing),
    #[br(pre_assert(name.starts_with("LINEDEF")))] LineDef(LineDef),
    #[br(pre_assert(name.starts_with("SIDEDEF")))] SideDef(SideDef),
    #[br(pre_assert(name.starts_with("VERTEX")))] Vertex(Vertex),
    #[br(pre_assert(name.starts_with("SEG")))] Segment(Segment),
    #[br(pre_assert(name.starts_with("SSECTOR")))] SubSector(SubSector),
    #[br(pre_assert(name.starts_with("NODE")))] Node(Node),
    #[br(pre_assert(name.starts_with("SECTOR")))] Sector(Sector),
    #[br(pre_assert(name.starts_with("REJECT")))] Reject(Reject) ,
    #[br(pre_assert(name.starts_with("BLOCKMAP")))] BlockMap(BlockMap),
    #[default] N
}

try_outer_to_inner!(DeserializeLump, Thing, Error, Unwraping);
try_outer_to_inner!(DeserializeLump, LineDef, Error, Unwraping);
try_outer_to_inner!(DeserializeLump, SideDef, Error, Unwraping);
try_outer_to_inner!(DeserializeLump, Vertex, Error, Unwraping);
try_outer_to_inner!(DeserializeLump, Segment, Error, Unwraping);
try_outer_to_inner!(DeserializeLump, SubSector, Error, Unwraping);
try_outer_to_inner!(DeserializeLump, Node, Error, Unwraping);
try_outer_to_inner!(DeserializeLump, Sector, Error, Unwraping);
try_outer_to_inner!(DeserializeLump, Reject, Error, Unwraping);
try_outer_to_inner!(DeserializeLump, BlockMap, Error, Unwraping);


#[derive(Debug, BinRead, Clone, PartialEq, Eq)]

pub struct Thing {
    pub x: i16,
    pub y: i16,
    pub angle_facing: i16,
    pub doomed_thing_type: i16,
    pub flags: ThingFlags,
}

#[derive(Debug, BinRead, Clone, PartialEq, Eq)]
pub struct LineDef {
    pub start_vertex_id: i16,
    pub end_vertex_id: i16,
    pub flags: LineDefFlags,
    pub special_type: i16,
    pub tag: i16,
    pub front: i16,
    pub back: i16,
}

#[derive(Debug, BinRead, Clone, PartialEq, Eq)]
pub struct SideDef {
    pub x_offset: i16,
    pub y_offset: i16,
    #[br(count = 8, map = |x: Vec<u8>| bytes_to_string(x))]
    pub name_of_upper: String,
    #[br(count = 8, map = |x: Vec<u8>| bytes_to_string(x))]
    pub name_of_lower: String,
    #[br(count = 8, map = |x: Vec<u8>| bytes_to_string(x))]
    pub name_of_middle: String,
    pub sector_this_sidedef_faces: i16
}

#[derive(Debug, BinRead, Clone, PartialEq, Eq)]
pub struct Vertex {
    pub x: i16,
    pub y: i16
}

#[derive(Debug, BinRead, Clone, PartialEq, Eq)]
pub struct Segment {
    pub start_vertext_id: i16,
    pub end_verext_id: i16,
    pub angle: i16, // full circle is -32768 to 32767.
    pub line_def_id: i16,
    pub direction: SegDirection,
    pub offset: i16, // distance along linedef to start of segments 
}


#[derive(Debug, BinRead, Clone, PartialEq, Eq)]
pub struct SubSector {
    pub segments_count: i16,
    pub first_segments_id: i16
}

impl SubSector {
    pub fn to_range(&self) -> core::ops::RangeInclusive<usize> {
        // Might need to be non inclusive
        (self.first_segments_id as usize)..=(self.segments_count as usize)
    }
}

#[derive(Debug, BinRead, Clone, PartialEq, Eq)]
pub struct Node {
    pub x_partion: i16,
    pub y_partion: i16,
    pub dx_partion: i16,
    pub dy_partion: i16, 
    pub front_bbox: BoundingBox,
    pub back_bbox: BoundingBox,
    pub right_child: i16,
    pub left_child: i16,
}


#[derive(Debug, BinRead, Clone, PartialEq, Eq)]
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
}

#[derive(Debug, BinRead, Clone, PartialEq, Eq)]
pub struct Reject {
    #[br(count = reject_count())]
    pub table: Vec<u8>
}

#[derive(Debug, BinRead, Clone, PartialEq, Eq)]
pub struct BlockMap {
    pub x_grid_origin: i16,
    pub y_grid_origin: i16,
    pub columns: i16,
    pub rows: i16,
    //#[br(count = (columns as usize * rows as usize))]
    //pub offsets: Vec<i16>
}

#[derive(Debug, BinRead, Clone, PartialEq, Eq)]
pub enum SegDirection {
    #[br(magic = b"0")] SameAsLineDef,
    #[br(magic = b"1")] OppositOfLineDef,
}

#[derive(Debug, BinRead, Clone, PartialEq, Eq)]
pub struct BoundingBox {
    pub x: i16,
    pub h: i16,
    pub y: i16,
    pub w: i16,
}

#[bitfield]
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
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