#![allow(unused_imports)]
mod lumps;
mod structure;
mod errors;

use crate::cli;


pub use errors::*;
pub use structure::*;
pub use lumps::{
    Lump, 
    ThingFlags,
    LineDefFlags,
    Thing,
    LineDef,
    SideDef,
    Vertex,
    Segment,
    SubSector,
    Node,
    BoundingBox,
    Sector,
    Reject,
    BlockMap,
};

use std::collections::HashMap;
use std::string::ToString;
use std::path::PathBuf;
use std::fs;
use std::io::{self, prelude::*, SeekFrom};

use std::convert::From;    // for sd in map_data.side_defs.lump_data_deserialized().iter() {
    //     println!("{sd:?}");
    // }



//use binread::{BinRead, io::Cursor, BinResult, ReadOptions};
use binrw::{
    binrw,    // #[binrw] attribute
    BinRead,  // trait for reading
    BinWrite, // trait for writing
    BinReaderExt, 
    io::Cursor,
};



pub type Wads = HashMap<String, Wad>;
pub type RawData = Vec<u8>;

#[derive(Debug)]
pub struct Wad {
    pub path: PathBuf,
    pub meta: WadMeta,
    pub raw_data: RawData,
}

#[derive(Debug)]
pub struct Reader {
    pub wads: Wads, 
}

impl Reader {
    pub fn new(args: &cli::Args) -> CliResult<Self> {
        let wads: Wads = 
            args.wad_paths.iter().try_fold(Wads::new(), |mut wads, path| {
                let name = path.file_stem()
                                        .ok_or_else(|| Error::FilePath(path.clone()))?
                                        .to_str().unwrap()
                                        .to_string();

                let mut data = fs::read(path)
                    .or_else(|err| Err(Errors::from(Error::FileOpen(err.to_string()))))?;
                
                wads.insert(
                    name,
                    Wad {
                        path: path.clone(),
                        meta: WadMeta::new(&data),
                        raw_data: data,
                        
                    }
                );
                Ok::<Wads, Errors>(wads)
            })?;
        Ok(Self {
            wads,
        })

    }

    /// Return a list of all MapMarker lumps within the given WAD directory
    pub fn get_map_list<'a, 'b, 'c>(&'a self, wad_name: &'b str) -> CliResult<'c, Vec<&Lump>> {
          Ok(self.lumps_for(wad_name)?.iter().filter(|l| l.kind == lumps::LumpKind::MapMarker ).collect())
    }
    /// Return all Lumps for a given wad
    pub fn lumps_for<'a, 'b, 'c>(&'a self, wad_name: &'b str) -> CliResult<'c, &Vec<Lump>> {
        Ok(&self.wads.get(wad_name).ok_or_else(|| Error::Reader(format!("'{wad_name}' not found")))?.meta.lumps)
    }

    /// Return a Map struct for a given WAD & map_name
    pub fn get_map<'a, 'b, 'c>(&'a self, wad_name: &'b str, map_name: &'b str) -> CliResult<'c, Map>  {
        let wad = &self.wads.get(wad_name).ok_or_else(|| Error::Reader(format!("'{wad_name}' not found")))?;
        let (i, _) = wad.meta.lumps.iter().enumerate().find(|(_, lump)| lump.name.starts_with(map_name))
        .ok_or_else(|| Error::Reader(format!("'{map_name}' not found")))?;
        Ok( Map::new(&wad.meta.lumps, &wad.raw_data, i))
    }

}

