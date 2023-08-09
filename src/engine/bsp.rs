use core::cmp::PartialOrd;
use super::*;

#[derive(Debug)]
pub struct Tree {
    pub nodes: wad::Lump,
    pub sub_sectors: wad::Lump,
    pub segments: wad::Lump,
    pub root_node_id: usize,
}

//TODO: Fix using .clone()
impl Tree {
    pub fn new(map: &wad::Map) -> Self {
        //nodes: wad::Lump, sub_sectors: wad::Lump, segments: wad::Lump
        let nodes = map.nodes.clone();
        let sub_sectors = map.sub_sectors.clone();
        let segments = map.segments.clone();
        let root_node_id = nodes.lump_data_deserialized().len() - 1;
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
}


#[derive(Debug, PartialOrd, Ord, PartialEq, Eq)]
pub struct Node<T: PartialOrd> {
    pub value: T,
    pub left: NodeType<T>,
    pub right: NodeType<T>,

}

/// Node -> Node -> Node -> SubSector -> [Segment, Segment, Segment]
/// 
#[derive(Default, Debug, PartialOrd, Ord, PartialEq, Eq )]
pub enum NodeType<T: PartialOrd> {
    Node(Box<Node<T>>),
    SubSector,
    #[default]
    None,

}

impl<T: PartialOrd> NodeType<T> {
    fn insert(&mut self, value: T) {
        match self {
            Self::Node(node) => node.insert(value),
            _ => ()
        };
    }

    fn traverse(&self, value: &T) {
        match self {
            Self::Node(node) => node.traverse(value),
            _ => ()
        };
    }
}

impl<T: PartialOrd> Node<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            left: NodeType::None,
            right: NodeType::None,
        }
    }
    pub fn insert(&mut self, value: T) {
        if value < self.value {
            if self.left == NodeType::None {
                self.left = NodeType::Node(Box::new(Node::new(value)))
            } else {
                self.left.insert(value)
            }
        } else if value > self.value {
            if self.right == NodeType::None {
                self.right = NodeType::Node(Box::new(Node::new(value)))
            } else {
                self.right.insert(value)
            }
        }
    }

    pub fn traverse(&self, player_pos: &T) {
        if player_pos <= &self.value {
            self.left.traverse(&player_pos);
            self.right.traverse(&player_pos);
        } else {
            self.right.traverse(&player_pos);
            self.left.traverse(&player_pos);
        }
    }
}