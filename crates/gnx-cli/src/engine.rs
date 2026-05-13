use gnx_core::graph::ArchivedZeroCopyGraph;
use memmap2::Mmap;
use rkyv::rancor::Error;
use std::fs::File;
use std::path::Path;

pub struct Engine {
    mmap: Mmap,
}

impl Engine {
    pub fn load<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let file = File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };
        Ok(Self { mmap })
    }

    pub fn graph(&self) -> Result<&ArchivedZeroCopyGraph, Error> {
        rkyv::access::<ArchivedZeroCopyGraph, Error>(&self.mmap)
    }
}
