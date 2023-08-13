use core::cmp::PartialOrd;
use std::iter::Iterator;
use crate::wad::{Node, SubSector};

use super::*;

#[derive(Debug)]
pub struct Tree {
    pub nodes: wad::Lump,
    pub sub_sectors: wad::Lump,
    pub segments: wad::Lump,
    pub root_node_id: i16,
}

//TODO: Fix using .clone()
impl Tree {
    pub fn new(map: &wad::Map) -> Self {
        //nodes: wad::Lump, sub_sectors: wad::Lump, segments: wad::Lump
        let nodes = map.nodes.clone();
        let sub_sectors = map.sub_sectors.clone();
        let segments = map.segments.clone();
        let root_node_id = (nodes.data.len() - 1) as i16;
        Self {
            nodes,
            sub_sectors,
            segments,
            root_node_id,
        }
    }
    pub fn update() {
        todo!();
    }

    pub fn traverse<Returned>(&self, thing_pos: (i16, i16)) -> TreeTraverseIterator<Returned> {
        TreeTraverseIterator::new(&self, self.root_node_id, thing_pos)
    }

    pub fn traverse_from<Returned>(&self, node_id: i16, thing_pos: (i16, i16)) -> TreeTraverseIterator<Returned> {
        TreeTraverseIterator::new(&self, node_id, thing_pos)
    }
}

pub struct TreeTraverseIterator<'t, Returned=Node> {
    tree: &'t Tree,
    thing_pos: (i16, i16),
    current_node: i16,
    finished: bool,
    _return: PhantomData<Returned>
}

impl<'t, Returned> TreeTraverseIterator<'t, Returned> {
    const SUB_SECTOR_IDENTIFIER: u16 = 0x8000;

    pub fn new(tree: &'t Tree, current_node: i16, thing_pos: (i16, i16)) -> Self {
        Self {
            tree,
            thing_pos,
            current_node,
            finished: false,
            _return: PhantomData::default()
        }
    }

    pub fn is_in_back_side(&self, node: &wad::Node, (player_x, player_y): (i16, i16)) -> bool {
        let dx = (player_x - node.x_partion) as i32;
        let dy = (player_y - node.y_partion) as i32;
        dx * node.dy_partion as i32 - dy * node.dx_partion as i32 <= 0
    }
}

impl<'t> Iterator for TreeTraverseIterator<'t, Node> {
    type Item = Node;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished { return None};

        if self.current_node as u16 >= Self::SUB_SECTOR_IDENTIFIER {
            None
        } else {
            let node: &wad::Node = self.tree.nodes.lump_data_deserialized().get(self.current_node  as usize).unwrap().try_into().unwrap();
            self.current_node = if self.is_in_back_side(node, self.thing_pos) {
                node.back_child_id
            } else {
                node.front_child_id
            };
            Some(node.clone())
        }
    }
}

impl<'t> Iterator for TreeTraverseIterator<'t, NodeType> {
    type Item = NodeType;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished { return None};

        if self.current_node as u16 >= Self::SUB_SECTOR_IDENTIFIER {
            self.finished = true;
            let leaf: &wad::SubSector = self.tree.sub_sectors.lump_data_deserialized().get((self.current_node as u16 - Self::SUB_SECTOR_IDENTIFIER) as usize).unwrap().try_into().unwrap();
            Some(NodeType::SubSector(leaf.clone()))
        } else {
            let node: &wad::Node = self.tree.nodes.lump_data_deserialized().get(self.current_node  as usize).unwrap().try_into().unwrap();
            self.current_node = if self.is_in_back_side(node, self.thing_pos) {
                node.back_child_id
            } else {
                node.front_child_id
            };
            Some(NodeType::Node(node.clone()))
        }
    }
}

pub enum NodeType {
    Node(wad::Node),
    SubSector(wad::SubSector),
}