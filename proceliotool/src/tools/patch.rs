use std::path::{Path, PathBuf};
use std::str::FromStr;
use itertools::Itertools;
use serde::{Serialize, Deserialize};
use md5::Digest;
use super::diff::DeltaManifest;
use std::io::Read;

pub struct PatchTool {

}

impl super::ProcelioCLITool for PatchTool {
    fn command(&self) -> &'static str {
        "patch"
    }
    
    fn usage(&self) {
      println!("path/to/files path/to/patch");
      println!("    Applies patch in-place");
    }

    fn tool(&self, args: Vec<String>) {
        tool_impl(args)
    }
}

#[derive(Serialize, Deserialize)]
pub struct InstallManifest {
  pub version: String,
  pub channel: String,
  pub exec: String
}

pub fn patch_bytes(from: &[u8], patch: &[u8]) -> Vec<u8> {
  vcdiff::decode(from, patch)
}

pub fn check_rollback(dir: &std::path::PathBuf) -> Result<(), anyhow::Error> {
  let tmpflag = dir.join("rollback");
  println!("Checking for rollback file {}: {}", tmpflag.display(), tmpflag.is_file());
  if !tmpflag.is_file() {
      return Ok(());
  }
  let rollback = std::fs::read_to_string(&tmpflag)?;

  let mut to_rename = Vec::new();
  for entry in walkdir::WalkDir::new(dir) {
      let entry = entry?;
      if let Some(x) = entry.path().extension().and_then(|x|x.to_str())
          && x.ends_with(&rollback) {
              to_rename.push(entry.path().to_owned());
          }
  }

  for entry in to_rename {
    std::fs::rename(&entry,  entry.with_extension(""))?;
  }
  std::fs::remove_file(tmpflag)?;
  Ok(())
}

pub fn patch(manifest: DeltaManifest, root_path: &std::path::PathBuf, rollback: String, files: impl Iterator<Item = (std::path::PathBuf, Vec<u8>)>, cb: Option<&dyn Fn(f32, String)>) -> Result<(), std::io::Error> {
  let tmpflag = root_path.join("rollback");
  std::fs::write(&tmpflag, &rollback)?;

  let patch_str = std::ffi::OsString::from_str("patch").unwrap();
  let roll_str = std::ffi::OsString::from_str(&rollback).unwrap();
  let mut i = 0;
  let n = manifest.hashes.len();
  for (rawpath, data) in files {
      let path = root_path.join(&rawpath);
      if path.is_dir() || path == tmpflag {
        i += 1;
        continue;
      }

      if let Some(callback) = cb {
        callback((i as f32) / (n as f32), format!("Patching {}", &rawpath.display()));
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
      i += 1;
  }

  i = 0;

  for elem in manifest.hashes {
    let mut data = elem.split(':');
    let hash = data.next().unwrap().to_ascii_lowercase();
    let file = data.join(":");
    if let Some(callback) = cb {
      callback((i as f32) / (n as f32), format!("Verifying {}", &file));
    }
    let mut md5hash = md5::Md5::new();
    std::io::copy(&mut std::fs::File::open(root_path.join(&file))?, &mut md5hash)?;
    let res = md5hash.finalize();
    let encoded = hex::encode(res).to_ascii_lowercase();
    if encoded != hash {
      println!("{}: {}  {}", file, encoded, hash);
      return Err(std::io::Error::from(std::io::ErrorKind::InvalidData));
    }
    i += 1;
  }

  if let Some(callback) = cb {
    callback(1., "Consolidating".to_string());
  }

  for elem in manifest.delete {
    let path = root_path.join(elem);
    let moved = path.with_extension(format!("{}.{}", path.extension().map(|x|x.to_string_lossy().into_owned()).unwrap_or("".to_owned()), &rollback));
    std::fs::rename(path, moved)?;
  }

  let gamemanifest = InstallManifest {
    exec: manifest.new_exec,
    version: manifest.target.version.clone(),
    channel: manifest.target.channel.clone()
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

  let mut i = 0;
  let n = to_del.len();
  for path in to_del {
    if let Some(callback) = cb {
      callback((i as f32) / (n as f32), "Cleaning Up".to_string());
    }
    std::fs::remove_file(path)?;
    i += 1;
  }
  
  std::fs::remove_file(tmpflag)?;
  Ok(())
}



pub fn from_dir(src_path: PathBuf, patch_path: PathBuf) {
  let manifest: DeltaManifest = serde_json::from_str(&std::fs::read_to_string(patch_path.join("manifest.json")).unwrap()).unwrap();

  let iter = walkdir::WalkDir::new(&patch_path).into_iter()
    .flatten()
    .filter(|x|!x.path().ends_with("manifest.json"))
    .filter(|x|!x.path().is_dir())
    .map(|x| {
      println!("{}", x.path().display());
      (x.path().strip_prefix(&patch_path).unwrap().to_owned(), std::fs::read(src_path.join(x.path())).unwrap())
    });

    println!("{:?}", patch(manifest, &src_path, "ROLLBACK".to_owned(), iter, None));//.unwrap();
}

use std::io::{Seek, BufRead};
pub fn from_zip<T: Seek + BufRead>(src_path: PathBuf, patch_data: &mut zip::ZipArchive<T>, cb: Option<&dyn Fn(f32, String)>) -> Result<(), anyhow::Error> {
  let count = patch_data.len();
  if count == 0 {
    return Err(anyhow::Error::msg("EMPTY ZIP"));
  }

  let manifest: DeltaManifest = {
    let mut curs = std::io::Cursor::new(Vec::new());
    std::io::copy(&mut patch_data.by_index(0)?, &mut curs)?;
    serde_json::from_slice(&curs.into_inner()).map_err(|_|anyhow::Error::msg("Invalid zip manifest format"))?
  };

  let iter = (1..count).filter_map(|x| {
    let mut d = patch_data.by_index(x).unwrap();
    if d.is_file() {
      let mut v: Vec<u8> = Vec::new();
      d.read_to_end(&mut v).unwrap();
      d.enclosed_name().map(|s| (src_path.join(s), v))
    } else {
      None
    }
  });
  let err_count = 0;

  println!("{:?}", patch(manifest, &src_path, "ROLLBACK".to_owned(), iter, cb)?);
  if err_count != 0 {
    Err(anyhow::anyhow!("Encountered errors unzipping zip"))
  } else {
    Ok(())
  }
}

fn tool_impl(args: Vec<String>) {
  let mut args = args.into_iter();
  let from = args.next().unwrap();
  let patsch = args.next().unwrap();

  let src_path = Path::new(&from).canonicalize().unwrap();
  let patch_path = Path::new(&patsch).canonicalize().unwrap();
    
  println!("{:?}", check_rollback(&src_path));

  if patch_path.is_dir() {
    from_dir(src_path, patch_path);
  } else if patch_path.is_file() && patch_path.extension().unwrap().to_string_lossy() == "zip" {
    let mut zip = zip::ZipArchive::new(Box::new(std::io::BufReader::new(std::fs::File::open(patch_path).unwrap()))).unwrap();
    from_zip(src_path, &mut zip, None).unwrap();
  }
}