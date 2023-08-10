#![allow(unused_imports)]

use std::{collections::{HashMap, BTreeMap}, hash::Hash, marker::PhantomData};

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

use std::sync::OnceLock;
pub type Point = (i16, i16);
pub type Points = Vec<Point>;

pub type P1P2 = (Point, Point);

pub type LineDefVertexes = Vec<P1P2>;


#[derive(Debug)]
pub struct Map {
    pub name: String,
    pub things: Lump,
    pub line_defs: Lump,
    pub side_defs: Lump,
    pub vertexs: Lump,
    pub segments: Lump,
    pub sub_sectors: Lump,
    pub nodes: Lump,
    pub sectors: Lump,
    pub reject: Lump,
    pub block_map: Lump,
    map_points: OnceLock<Points>,
    map_bounds: OnceLock<P1P2>,
    line_defs_to_vertexes: OnceLock<Vec<((i16, i16), (i16, i16))>>,
}


type Offset = usize;

impl std::convert::From<(&Vec<Lump>, Offset)> for Map {
    fn from((wad_lumps, offset): (&Vec<Lump>, Offset)) -> Self {
        Self::new(wad_lumps, offset)
    }
}

impl Map {
    //TODO: Fix using .clone()
    pub fn new(wad_lumps: &Vec<Lump>, offset: Offset) -> Self {
        Map {
            name: wad_lumps[offset].name.clone(),
            things: wad_lumps[offset + 1].clone(),
            line_defs: wad_lumps[offset + 2].clone(),
            side_defs: wad_lumps[offset + 3].clone(),
            vertexs: wad_lumps[offset + 4].clone(),
            segments: wad_lumps[offset + 5].clone(),
            sub_sectors: wad_lumps[offset + 6].clone(),
            nodes: wad_lumps[offset + 7].clone(),
            sectors: wad_lumps[offset + 8].clone(),
            reject: wad_lumps[offset + 9].clone(),
            block_map: wad_lumps[offset + 10].clone(),
            map_points: OnceLock::new(),
            map_bounds: OnceLock::new(),
            line_defs_to_vertexes : OnceLock::new(),
        }
    }




    pub fn map_points(&self) -> &Points {
        &self.map_points.get_or_init(|| {
            self.vertexs.lump_data_deserialized().iter().fold(Points::new(), |mut points, v| {
                if let DeserializeLump::Vertex(vertex) = v {
                    points.push((vertex.x, vertex.y));
                    points
                } else { panic!("Tried to create map points from non Vertex Lump: `{v:?}`") }
                
            })
        })
    }

    pub fn map_bounds(&self) -> &P1P2 {
        self.map_bounds.get_or_init(|| {
            let points = self.map_points();
            let mut x_sorted = points.clone();
            x_sorted.sort_unstable_by_key(|v|  v.0 );
            let mut y_sorted = points.clone();
            y_sorted.sort_unstable_by_key(|v|  v.1 );
            ((x_sorted.first().unwrap().0, x_sorted.last().unwrap().0), (y_sorted.first().unwrap().1, y_sorted.last().unwrap().1))
        })

    }

    
    pub fn line_defs_to_vertexes<'a>(&'a self, map_points: Option<&'a Points>) -> &Vec<((i16, i16), (i16, i16))> {
        self.line_defs_to_vertexes.get_or_init(|| {
            let line_defs: &DeserializedLumps = &self.line_defs.lump_data_deserialized();
            let map_points: &Vec<(i16, i16)> = if let Some(m) = map_points { &m } else { self.map_points() };
            let mut output: Vec<((i16, i16), (i16, i16))> = Vec::new();

            for line_def in line_defs.iter() {
                let line_def: LineDef = line_def.into();
                output.push((map_points[line_def.start_vertex_id as usize], map_points[line_def.end_vertex_id as usize]));
            }
            output
        })
    }
}