#![allow(unused_imports)]

use std::{collections::{HashMap, BTreeMap}, hash::Hash};

use super::*;

use binrw::{BinReaderExt, binrw, BinRead, io::Cursor, args};

const DOOMMAPLUMPLENGTH: usize = 11;


pub type Lumps = Vec<Lump>;
#[derive(Debug, BinRead)]
#[br(little)]
pub struct WadData {
    pub identifaction: Identification,   
    pub lump_count: i32,
    pub dir_offset: i32,
    #[br(seek_before = SeekFrom::Start(dir_offset as u64), count = lump_count)]
    pub lumps: Lumps,
}


#[derive(Clone, Debug, PartialEq)]
#[binrw]
pub enum Identification {
    #[brw(magic = b"IWAD")] IWAD,
    #[brw(magic = b"PWAD")] PWAD,
} 

#[derive(Debug)]
pub struct Map<'m> {
    pub name: &'m str,
    pub things: &'m Lump,
    pub line_defs: &'m Lump,
    pub side_defs: &'m Lump,
    pub vertexs: &'m Lump,
    pub segments: &'m Lump,
    pub sub_sectors: &'m Lump,
    pub nodes: &'m Lump,
    pub sectors: &'m Lump,
    pub reject: &'m Lump,
    pub block_map: &'m Lump,
}