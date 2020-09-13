use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashSet;
use serde::{Serialize, Deserialize};
use super::build_stuff::*;
use md5::Digest;
use std::convert::TryFrom;



pub fn patch_file(from: &Path, patch: &Path, out: &Path) {
    let bytes_from = std::fs::read(from).unwrap();
    let writer = std::fs::File::create(out).unwrap();
    
    bsdiff_rs::jbspatch40_32bit(&bytes_from, writer, std::fs::File::open(patch).unwrap()).unwrap();
}

pub fn tool(mut args: std::env::Args) {
    let from = args.next().unwrap().to_owned();
  
    let patch = args.next().unwrap().to_owned();
  
    let patched = args.next().unwrap().to_owned();

    patch_file(Path::new(&from), Path::new(&patch), Path::new(&patched))
}