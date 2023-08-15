#![allow(unused_imports)]

use std::{collections::{HashMap, BTreeMap}, hash::Hash, marker::PhantomData};

use super::*;

use binrw::{BinReaderExt, binrw, BinRead, io::Cursor, args};

const DOOMMAPLUMPLENGTH: usize = 11;

pub type Lumps = Vec<Lump>;
#[derive(Debug, BinRead)]
#[br(little)]
pub struct WadMeta {
    pub identifaction: Identification,   
    pub lump_count: i32,
    pub dir_offset: i32,
    #[br(seek_before = SeekFrom::Start(dir_offset as u64), count = lump_count)]
    pub lumps: Lumps,
}

impl WadMeta {
    pub fn new(data: &RawData) -> Self {
        WadMeta::read(&mut Cursor::new(data)).unwrap()
    }

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
    pub things: Vec<Thing>,
    pub line_defs: Vec<LineDef>,
    pub side_defs: Vec<SideDef>,
    pub vertexes: Vec<Vertex>,
    pub segments: Vec<Segment>,
    pub sub_sectors: Vec<SubSector>,
    pub nodes: Vec<Node>,
    pub sectors: Vec<Sector>,
    pub reject: Vec<Reject>,
    pub block_map: Vec<BlockMap>,
    pub map_points: OnceLock<Points>,
    pub map_bounds: OnceLock<P1P2>,
    pub line_defs_to_vertexes: OnceLock<LineDefVertexes>,
}


type Offset = usize;

impl std::convert::From<(&Vec<Lump>, &RawData, Offset)> for Map {
    fn from((wad_lumps, raw_data, offset): (&Vec<Lump>, &RawData, Offset)) -> Self {
        Self::new(wad_lumps, raw_data, offset)
    }
}

pub struct SegsToDraw {
    pub sub_sector_id: u16,
    pub segments: Vec<Segment>
}

impl Map {
    //TODO: Fix using .clone()
    pub fn new(wad_lumps: &Vec<Lump>, raw_data: &RawData, offset: Offset) -> Self {
        Map {
            name: wad_lumps[offset].name.clone(),
            things: wad_lumps[offset + 1].deserialize(raw_data),
            line_defs: wad_lumps[offset + 2].deserialize(raw_data),
            side_defs: wad_lumps[offset + 3].deserialize(raw_data),
            vertexes: wad_lumps[offset + 4].deserialize(raw_data),
            segments: wad_lumps[offset + 5].deserialize(raw_data),
            sub_sectors: wad_lumps[offset + 6].deserialize(raw_data),
            nodes: wad_lumps[offset + 7].deserialize(raw_data),
            sectors: wad_lumps[offset + 8].deserialize(raw_data),
            reject: wad_lumps[offset + 9].deserialize(raw_data),
            block_map: wad_lumps[offset + 10].deserialize(raw_data),
            map_points: OnceLock::new(),
            map_bounds: OnceLock::new(),
            line_defs_to_vertexes : OnceLock::new(),
        }
    }




    pub fn map_points(&self) -> &Points {
        &self.map_points.get_or_init(|| {
            self.vertexes.iter().fold(Points::new(), |mut points, v| {
                    points.push((v.x, v.y));
                    points
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
            let map_points: &Vec<(i16, i16)> = if let Some(m) = map_points { &m } else { self.map_points() };
            let mut output: Vec<((i16, i16), (i16, i16))> = Vec::new();

            for line_def in self.line_defs.iter() {
                output.push((map_points[line_def.start_vertex_id as usize], map_points[line_def.end_vertex_id as usize]));
            }
            output
        })
    }
    
    pub fn segs_from_nodes(&self, nodes: &Vec<Node>, player_pos: (i16, i16)) -> Vec<SegsToDraw> {
        nodes.iter().rev().fold( (None, Vec::new()), |(previous_node, segments): (Option<&Node>, Vec<SegsToDraw>), node: &Node| {
            println!("{node:?}");
            (
                Some(node),
                self.recurse_node_from_list(segments, node, previous_node, player_pos)
            )
        }).1
    }

    #[inline]
    fn recurse_node_from_list(&self, mut segments: Vec<SegsToDraw>, node: &Node, previous_node: Option<&Node>, player_pos: (i16, i16)) -> Vec<SegsToDraw> {
        let (b_is_ssector, f_is_ssector) = node.children_are_sub_sectors();
        if node.is_in_back_side(player_pos) {
            if previous_node == None || previous_node.is_some_and(|pn| node.back_child_id != pn.id) {
                self.push_or_traverse_child(&mut segments, b_is_ssector, node.back_child_id, player_pos);
            }
            if previous_node == None || previous_node.is_some_and(|pn| node.front_child_id != pn.id) {
                self.push_or_traverse_child(&mut segments, f_is_ssector, node.front_child_id, player_pos);   
            }
        } else {
            if  previous_node == None || previous_node.is_some_and(|pn| node.front_child_id != pn.id) {
                self.push_or_traverse_child(&mut segments, f_is_ssector, node.front_child_id, player_pos);   
            }
            if  previous_node == None || previous_node.is_some_and(|pn| node.back_child_id != pn.id) {
                self.push_or_traverse_child(&mut segments, b_is_ssector, node.back_child_id, player_pos);
            }
        }
        segments
    }

    fn push_or_traverse_child(&self, segments: &mut Vec<SegsToDraw>, is_ssector: bool, child_id: u16, player_pos: (i16, i16)) {
        if is_ssector { 
            segments.push(SegsToDraw {
                sub_sector_id: child_id,
                segments: self.get_segments(self.sub_sectors[(child_id & SubSector::IDENTIFIER_BITMASK) as usize]) 
            });
        } else {
            self.recurse_subtree(segments, &self.nodes[child_id as usize], player_pos);
        }
    }

    fn recurse_subtree(&self, segments: &mut Vec<SegsToDraw>,  node: &Node, player_pos: (i16, i16)){
        let (b_is_ssector, f_is_ssector) = node.children_are_sub_sectors();
        if node.is_in_back_side(player_pos) {
            self.push_or_traverse_child(segments, b_is_ssector, node.back_child_id, player_pos);
            self.push_or_traverse_child(segments, f_is_ssector, node.front_child_id, player_pos);   
        } else {
            self.push_or_traverse_child(segments, f_is_ssector, node.front_child_id, player_pos); 
            self.push_or_traverse_child(segments, b_is_ssector, node.back_child_id, player_pos);
        }
    }

    pub fn get_segments(&self, ss: SubSector) -> Vec<Segment> {
        ss.to_range().map(|id| self.segments[id] ).collect()
    }

    pub fn traverse_bsp<Returned>(&self, thing_pos: (i16, i16)) -> TreeTraverseIterator<Returned> {
        TreeTraverseIterator::new(&self.nodes, (self.nodes.len() - 1) as u16, thing_pos)
    }

    pub fn traverse_bsp_from<Returned>(&self, node_id: u16, thing_pos: (i16, i16)) -> TreeTraverseIterator<Returned> {
        TreeTraverseIterator::new(&self.nodes, node_id, thing_pos)
    }
}

pub type Nodes = Vec<Node>;
pub struct TreeTraverseIterator<'t, Returned=Node> {
    nodes: &'t Vec<Node>,
    thing_pos: (i16, i16),
    current_node_id: u16,
    finished: bool,
    _return: PhantomData<Returned>
}

impl<'t, Returned> TreeTraverseIterator<'t, Returned> {
    pub fn new(nodes: &'t Nodes, current_node_id: u16, thing_pos: (i16, i16)) -> Self {
        Self {
            nodes,
            thing_pos,
            current_node_id,
            finished: false,
            _return: PhantomData::default()
        }
    }
}

impl<'t> Iterator for TreeTraverseIterator<'t, Node> {
    type Item = Node;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished { return None};

        if self.current_node_id as u16 >= SubSector::SUB_SECTOR_IDENTIFIER {
            None
        } else {
            let node = self.nodes.get(self.current_node_id  as usize).unwrap();
            self.current_node_id = if node.is_in_back_side(self.thing_pos) {
                node.back_child_id
            } else {
                node.front_child_id
            };
            Some(node.clone())
        }
    }
}

// impl<'t> Iterator for TreeTraverseIterator<'t, NodeType> {
//     type Item = NodeType;

//     fn next(&mut self) -> Option<Self::Item> {
//         if self.finished { return None};

//         if self.current_node_id as u16 >= Self::SUB_SECTOR_IDENTIFIER {
//             self.finished = true;
//             let leaf = self.sub_sectors.lump_data_deserialized().get((self.current_node_id as u16 - Self::SUB_SECTOR_IDENTIFIER) as usize).unwrap().try_into().unwrap();
//             Some(NodeType::SubSector(leaf.clone()))
//         } else {
//             let node = self.nodes.lump_data_deserialized().get(self.current_node_id  as usize).unwrap().try_into().unwrap();
//             self.current_node_id = if self.is_in_back_side(node, self.thing_pos) {
//                 node.back_child_id
//             } else {
//                 node.front_child_id
//             };
//             Some(NodeType::Node(node.clone()))
//         }
//     }
// }

// pub enum NodeType {
//     Node(wad::Node),
//     SubSector(wad::SubSector),
// }