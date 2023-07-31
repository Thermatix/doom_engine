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
pub type Point = (i32, i32);
pub type Points = Vec<Point>;

pub type P1P2 = (Point, Point);

pub type LineDefVertexes = Vec<(P1P2)>;


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
    map_points: OnceLock<Points>,
    map_bounds: OnceLock<P1P2>,
}


// TODO: Impl From<(Vec<Lump>, usize)>

impl<'m> Map<'m> {

    pub fn new(wad_lumps: &'m Vec<Lump>, offset: usize) -> Self {
        Map {
            name: &wad_lumps[offset].name,
            things: &wad_lumps[offset + 1],
            line_defs: &wad_lumps[offset + 2],
            side_defs: &wad_lumps[offset + 3],
            vertexs: &wad_lumps[offset + 4],
            segments: &wad_lumps[offset + 5],
            sub_sectors: &wad_lumps[offset + 6],
            nodes: &wad_lumps[offset + 7],
            sectors: &wad_lumps[offset + 8],
            reject: &wad_lumps[offset + 9],
            block_map: &wad_lumps[offset + 10],
            map_points: OnceLock::new(),
            map_bounds: OnceLock::new(),
        }
    }


    pub fn scale_map_points(&self, max_width: i32, max_height: i32, boarder: i32) -> Points {
        let ((x_min, x_max),(y_min, y_max)) = self.map_bounds();
        self.map_points().iter().map(|(x, y)| {
            (
                Self::scale_x(*x_min, *x_max, *x, boarder, max_width - boarder),
                Self::scale_y(*y_min, *y_max, *y, boarder, max_height - boarder, max_height)
            )
        }).collect()
    } 

    #[inline]
    fn scale_x(x_min: i32, x_max: i32, n: i32, out_min: i32, out_max: i32) -> i32 {
        (x_min.max(x_max.min(n)) - x_min) * (out_max - out_min) / (x_max - x_min) + out_min
    }

    fn scale_y(y_min: i32, y_max: i32, n: i32, out_min: i32, out_max: i32, max_height: i32) -> i32 {
        max_height - (y_min.max(y_max.min(n)) - y_min) * (out_max - out_min) / (y_max - y_min) - out_min
    }

    pub fn map_points(&self) -> &Points {
        &self.map_points.get_or_init(|| {
            self.vertexs.lump_data_deserialized().iter().fold(Points::new(), |mut points, v| {
                if let DeserializeLump::Vertex { x, y } = v {
                    points.push((*x as i32, *y as i32));
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

    pub fn line_defs_to_vertexes(&self, map_points: Option<&Points>) -> LineDefVertexes {
        if let LumpData::DeserializeLump(line_defs) = &self.line_defs.data {
            let map_points = if let Some(m) = map_points { m } else { self.map_points() };
            line_defs.iter().map(|line_def| {
                if let DeserializeLump::LineDef { start_vertex_id, end_vertex_id, flags, special_type, tag, front, back } = &line_def {
                    (map_points[*start_vertex_id as usize], map_points[*end_vertex_id as usize])  
                } else { panic!("line_defs are not LINEDEFS")}
            }).collect()
        } else { panic!("line_defs are not LINEDEFS")}
    }
}