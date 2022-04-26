use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashSet;
use std::str::FromStr;
use itertools::Itertools;
use serde::{Serialize, Deserialize};
use super::build_stuff::*;
use md5::Digest;
use std::convert::TryFrom;
use super::diff::DeltaManifest;

#[derive(Serialize, Deserialize)]
pub struct InstallManifest {
    pub exec: String,
    pub dev: bool,
    pub version: Vec<i32>,
}

pub fn patch_bytes(from: &[u8], patch: &[u8]) -> Vec<u8> {
  vcdiff::decode(from, patch)
}

pub fn patch_file(from: &Path, patch: &Path, out: &Path) {
  let bytes_from = std::fs::read(from).unwrap();
  let bytes_patch = std::fs::read(patch).unwrap();
  std::fs::write(out, &patch_bytes(&bytes_from, &bytes_patch)).unwrap();
}

pub fn check_rollback(dir: &std::path::PathBuf) -> Result<(), anyhow::Error> {
  let tmpflag = dir.join("rollback");
  if !tmpflag.is_file() {
      return Ok(());
  }
  let rollback = std::fs::read_to_string(&tmpflag)?;

  let mut to_rename = Vec::new();
  for entry in walkdir::WalkDir::new(&dir) {
      let entry = entry?;
      if let Some(x) = entry.path().extension().and_then(|x|x.to_str()) {
          if x.ends_with(&rollback) {
              to_rename.push(entry.path().to_owned());
          }
      }
  }

  for entry in to_rename {
    std::fs::rename(&entry,  entry.with_extension(""))?;
  }
  std::fs::remove_file(tmpflag)?;
  Ok(())
}

pub fn patch(manifest: DeltaManifest, root_path: &std::path::PathBuf, rollback: String, files: impl Iterator<Item = (std::path::PathBuf, Vec<u8>)>) -> Result<(), std::io::Error> {
  let tmpflag = root_path.join("rollback");
  std::fs::write(&tmpflag, &rollback)?;

  let patch_str = std::ffi::OsString::from_str("patch").unwrap();
  let roll_str = std::ffi::OsString::from_str(&rollback).unwrap();
  for (path, data) in files {
      let path = root_path.join(path);
      if path.is_dir() || path == tmpflag {
        continue;
      }

      if path.extension() == Some(&patch_str) {
        let rollback_path = path.with_extension(&rollback);
        let real_path = path.with_extension("");

        let file_bytes = std::fs::read(&real_path)?;
        let patched = patch_bytes(&file_bytes, &data);
        std::fs::rename(&real_path, rollback_path)?;
        std::fs::write(real_path, patched)?;
      } else {
        if let Some(s) = path.parent() {
          std::fs::create_dir_all(s)?;
        }
        std::fs::write(path, &data)?;
      };
  }

  for elem in manifest.hashes {
    let mut data = elem.split(':');
    let hash = data.next().unwrap().to_ascii_lowercase();
    let file = data.join(":");
    
    let mut md5hash = md5::Md5::new();
    std::io::copy(&mut std::fs::File::open(root_path.join(file))?, &mut md5hash)?;
    let res = md5hash.finalize();
    if hex::encode(res).to_ascii_lowercase() != hash {
      return Err(std::io::Error::from(std::io::ErrorKind::InvalidData));
    }
  }

  for elem in manifest.delete {
    let path = root_path.join(elem);
    let moved = path.with_extension(format!("{}.{}", path.extension().map(|x|x.to_string_lossy().into_owned()).unwrap_or("".to_owned()), &rollback));
    std::fs::rename(path, moved)?;
  }

  let gamemanifest = InstallManifest {
    exec: manifest.newExec,
    version: vec!(manifest.target.major as i32, manifest.target.minor as i32, manifest.target.patch as i32),
    dev: manifest.target.dev_build
  };
  std::fs::rename(root_path.join("manifest.json"), root_path.join("manifest.json.a").with_extension(rollback))?;
  std::fs::write(root_path.join("manifest.json"), serde_json::to_string(&gamemanifest).unwrap())?;

  let mut to_del = Vec::new();
  for file in walkdir::WalkDir::new(root_path) {
    let f = file?.path().to_owned();
    if f.extension() == Some(&roll_str) {
      to_del.push(f);
    }
  }

  for path in to_del {
    std::fs::remove_file(path)?;
  }
  
  std::fs::remove_file(tmpflag)?;
  Ok(())
}

fn dump_usage() {
  println!("patch path/to/files path/to/patch");
  println!("  Applies patch in-place");
}

pub fn tool(mut args: std::env::Args) {
  let from = args.next().unwrap_or("--help".to_owned());
  if from == "--help" || from == "-h" {
      dump_usage();
      return;
  }
  let patsch = args.next().unwrap_or("--help".to_owned());
  if patsch == "--help" || patsch == "-h" {
      dump_usage();
      return;
  }

  let src_path = Path::new(&from).canonicalize().unwrap();
  let patch_path = Path::new(&patsch).canonicalize().unwrap();

  let manifest: DeltaManifest = serde_json::from_str(&std::fs::read_to_string(patch_path.join("manifest.json")).unwrap()).unwrap();
    
  println!("{:?}", check_rollback(&src_path));

  let iter = walkdir::WalkDir::new(&patch_path).into_iter()
    .filter(|x| x.is_ok())
    .map(|x|x.unwrap())
    .filter(|x|!x.path().ends_with("manifest.json"))
    .filter(|x|!x.path().is_dir())
    .map(|x| {
      println!("{}", x.path().display());
      (x.path().strip_prefix(&patch_path).unwrap().to_owned(), std::fs::read(src_path.join(x.path())).unwrap())
    });

    println!("{:?}", patch(manifest, &src_path, "ROLLBACK".to_owned(), iter));//.unwrap();
}