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
    pub sub_sector_id: i16,
    pub segments: Vec<Segment>
}

impl Map {
    //TODO: Fix using .clone()
    pub fn new(wad_lumps: &Vec<Lump>, raw_data: &RawData, offset: Offset) -> Self {
        Map {
            name: wad_lumps[offset].name.clone(),
            things: wad_lumps[offset + 1].deserialize::<Thing>(raw_data),
            line_defs: wad_lumps[offset + 2].deserialize::<LineDef>(raw_data),
            side_defs: wad_lumps[offset + 3].deserialize::<SideDef>(raw_data),
            vertexes: wad_lumps[offset + 4].deserialize::<Vertex>(raw_data),
            segments: wad_lumps[offset + 5].deserialize::<Segment>(raw_data),
            sub_sectors: wad_lumps[offset + 6].deserialize::<SubSector>(raw_data),
            nodes: wad_lumps[offset + 7].deserialize::<Node>(raw_data),
            sectors: wad_lumps[offset + 8].deserialize::<Sector>(raw_data),
            reject: wad_lumps[offset + 9].deserialize::<Reject>(raw_data),
            block_map: wad_lumps[offset + 10].deserialize::<BlockMap>(raw_data),
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

    pub fn segs_to_draw(&self, nodes: &Vec<Node>, player_pos: (i16, i16)) -> Vec<SegsToDraw> {
        let mut previous_node: Option<&Node>;
        nodes.iter().rev().fold(Vec::new(), |mut segments: Vec<SegsToDraw>, node: &Node| {
            if node.is_in_back_side(player_pos) {
                if node.child_is_sub_sector(node.back_child_id) {
                    if previous_node.is_some_and(|pn| pn.back_child_id != node.back_child_id ) {
                        segments.push(SegsToDraw {
                            sub_sector_id: node.back_child_id,
                            segments: self.get_segments(self.sub_sectors[node.back_child_id as usize]) 
                        });
                    }
                    if previous_node.is_some_and(|pn| pn.front_child_id != node.front_child_id ) {
                        segments.push(SegsToDraw {
                            sub_sector_id: node.front_child_id,
                            segments: self.get_segments(self.sub_sectors[node.back_child_id as usize]) 
                        });
                    }
                } else {
                    self.recurse_nodes(&mut segments, node.back_child_id, player_pos);
                    self.recurse_nodes(&mut segments, node.front_child_id, player_pos);
                }
            } else {
                if node.child_is_sub_sector(node.front_child_id) {
                    if previous_node.is_some_and(|pn| pn.front_child_id != node.front_child_id ) {
                        segments.push(SegsToDraw {
                            sub_sector_id: node.front_child_id,
                            segments: self.get_segments(self.sub_sectors[node.back_child_id as usize]) 
                        });

                    }
                    if previous_node.is_some_and(|pn| pn.back_child_id != node.back_child_id ) {
                        segments.push(SegsToDraw {
                            sub_sector_id: node.back_child_id,
                            segments: self.get_segments(self.sub_sectors[node.back_child_id as usize]) 
                        });
                    }
                    
                } else {
                    self.recurse_nodes(&mut segments, node.front_child_id, player_pos);
                    self.recurse_nodes(&mut segments, node.back_child_id, player_pos);
                }
            }

            previous_node = Some(&node);
            segments
        })
    }

    pub fn recurse_nodes(&self, mut segments: &mut Vec<SegsToDraw>, node_id: i16, player_pos: (i16, i16)) {
        let node = self.nodes[node_id as usize];
        if node.is_in_back_side(player_pos) {
            if node.child_is_sub_sector(node.back_child_id) {
                segments.push(SegsToDraw {
                    sub_sector_id: node.back_child_id,
                    segments: self.get_segments(self.sub_sectors[node.back_child_id as usize]) 
                });
                let ss = &self.sub_sectors[node.front_child_id as usize];
                segments.push(SegsToDraw {
                    sub_sector_id: node.front_child_id,
                    segments: self.get_segments(self.sub_sectors[node.back_child_id as usize]) 
                });
            } else {
                self.recurse_nodes(&mut segments, node.back_child_id, player_pos);
                self.recurse_nodes(&mut segments, node.front_child_id, player_pos);
            }
        } else {
            if node.child_is_sub_sector(node.front_child_id) {
                let ss = &self.sub_sectors[node.front_child_id as usize];
                segments.push(SegsToDraw {
                    sub_sector_id: node.front_child_id,
                    segments: self.get_segments(self.sub_sectors[node.back_child_id as usize]) 
                });

                segments.push(SegsToDraw {
                    sub_sector_id: node.back_child_id,
                    segments: self.get_segments(self.sub_sectors[node.back_child_id as usize]) 
                });
                
            } else {
                self.recurse_nodes(&mut segments, node.front_child_id, player_pos);
                self.recurse_nodes(&mut segments, node.back_child_id, player_pos);
            }
        }
    }

    pub fn get_segments(&self, ss: SubSector) -> Vec<Segment> {
        ss.to_range().map(|id| self.segments[id] ).collect()
    }

    pub fn traverse_bsp<Returned>(&self, thing_pos: (i16, i16)) -> TreeTraverseIterator<Returned> {
        TreeTraverseIterator::new(&self.nodes, (self.nodes.len() - 1) as i16, thing_pos)
    }

    pub fn traverse_bsp_from<Returned>(&self, node_id: i16, thing_pos: (i16, i16)) -> TreeTraverseIterator<Returned> {
        TreeTraverseIterator::new(&self.nodes, node_id, thing_pos)
    }
}

pub type Nodes = Vec<Node>;
pub struct TreeTraverseIterator<'t, Returned=Node> {
    nodes: &'t Vec<Node>,
    thing_pos: (i16, i16),
    current_node_id: i16,
    finished: bool,
    _return: PhantomData<Returned>
}

impl<'t, Returned> TreeTraverseIterator<'t, Returned> {
    const SUB_SECTOR_IDENTIFIER: u16 = 0x8000;

    pub fn new(nodes: &'t Nodes, current_node_id: i16, thing_pos: (i16, i16)) -> Self {
        Self {
            nodes,
            thing_pos,
            current_node_id,
            finished: false,
            _return: PhantomData::default()
        }
    }

    pub fn is_in_back_side(&self, node: &Node, (player_x, player_y): (i16, i16)) -> bool {
        let dx = (player_x - node.x_partion) as i32;
        let dy = (player_y - node.y_partion) as i32;
        dx * node.dy_partion as i32 - dy * node.dx_partion as i32 <= 0
    }
}

impl<'t> Iterator for TreeTraverseIterator<'t, Node> {
    type Item = Node;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished { return None};

        if self.current_node_id as u16 >= Self::SUB_SECTOR_IDENTIFIER {
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