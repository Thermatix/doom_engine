#![allow(unused_imports)]
mod lumps;
mod structure;
mod errors;

use crate::cli;


pub use errors::*;
pub use structure::*;
pub use lumps::{
    Lump, 
    DeserializedLumps, 
    DeserializeLump, 
    LumpData, 
    ThingFlags,
    LineDefFlags,
    Thing,
    LineDef,
    SideDef,
    Vertex,
    Segment,
    SubSector,
    Node,
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

#[derive(Debug)]
pub struct Wad {
    pub path: PathBuf,
    pub data: WadData,
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
                
            
                let mut reader = Cursor::new(&data);
                let headers_lumps = WadData::read(&mut reader).unwrap();
                wads.insert(
                    name,
                    Wad {
                        path: path.clone(),
                        data: headers_lumps,
                        
                    }
                );
                Ok::<Wads, Errors>(wads)
            })?;
        Ok(Self {
            wads,
        })

    }

    pub fn get_map_list<'a, 'b, 'c>(&'a self, wad_name: &'b str) -> CliResult<'c, Vec<&Lump>> {
        let lumps = &self.wads.get(wad_name)
            .ok_or_else(|| Error::Reader(format!("'{wad_name}' not found")))?
            .data.lumps;
        Ok(lumps.iter().filter(|l| l.data == lumps::LumpData::MapName ).collect())
    }

    pub fn lumps_for<'a, 'b, 'c>(&'a self, wad_name: &'b str) -> CliResult<'c, &Vec<Lump>> {
        Ok(&self.wads.get(wad_name).ok_or_else(|| Error::Reader(format!("'{wad_name}' not found")))?.data.lumps)
    }

    fn read_bytes(file: &mut fs::File, buffer: &mut [u8],  offset: usize, num_bytes: usize) -> Result<(), std::io::Error>{
        file.seek(SeekFrom::Start(offset as u64))?;
        let _ = file.read_exact(buffer);
        Ok(())
    }

    pub fn get_map<'a, 'b, 'c>(&'a self, wad_name: &'b str, map_name: &'b str) -> CliResult<'c, Map>  {
        let wad_lumps = &self.wads.get(wad_name).ok_or_else(|| Error::Reader(format!("'{wad_name}' not found")))?.data.lumps;
        let (i, _) = wad_lumps.iter().enumerate().find(|(_, lump)| lump.name.starts_with(map_name))
        .ok_or_else(|| Error::Reader(format!("'{map_name}' not found")))?;
        Ok((wad_lumps,i).into())
    }

}

