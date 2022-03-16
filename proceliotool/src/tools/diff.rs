use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashSet;
use serde::{Serialize, Deserialize};
use super::build_stuff::*;
use md5::Digest;
use std::convert::TryFrom;
#[derive(Serialize, Deserialize)]
pub struct DeltaManifest {
    pub hashes: Vec<String>,
    pub delete: Vec<String>,
    pub target: Version,
    pub source: Version,
    pub newExec: String
}

fn dump_usage() {
    println!("diff path/to/from path/to/to [path/to/patch]");
    println!("  Creates a patch from the 'from' directory to the 'to' directory");
}

#[cfg(not(windows))]
pub fn patch_file(from: &Path, to: &Path, patch: &Path) {

    let bytes_from = std::fs::read(from).unwrap();
    let bytes_to = std::fs::read(to).unwrap();
    if bytes_from == bytes_to {
        println!("File unchanged!");
        return;
    }
    println!("A {}", bytes_from.len());
    println!("B {}", bytes_to.len());

    let out = vcdiff::encode(&bytes_from, &bytes_to, vcdiff::FormatExtension::empty(), true);
    println!("D  {:?}", out.len());
    println!("C {:?}", &patch);
    std::fs::write(patch, out).unwrap();
}


#[cfg(windows)]
pub fn patch_file(from: &Path, to: &Path, patch: &Path) {
    println!("No patch generation from Windows, sorry");
    panic!();
}

fn get_all_files_recursive_impl(root: &Path, dir: &Path, map: &mut HashSet<PathBuf>) {
    let iter = fs::read_dir(dir).unwrap();
    for elem in iter {
        let path = elem.unwrap().path();
        if path.is_dir() {
            get_all_files_recursive_impl(root, &path, map);
        } else {
            let relative = path.strip_prefix(root).unwrap().to_path_buf();
            if relative != PathBuf::from("manifest.json") {
                println!("Found file {:?}", path);
                map.insert(relative);
            }
        }
    }
}

pub fn get_all_files_recursive(dir: &Path) -> HashSet<PathBuf> {
    let mut map = HashSet::new();
    get_all_files_recursive_impl(dir, dir, &mut map);
    map
}

pub fn extract_overlap(first: &mut HashSet<PathBuf>, second: &mut HashSet<PathBuf>) -> HashSet<PathBuf> {
    let mut both = HashSet::new();
    for elem in first.iter() {
        if second.contains(elem) {
            both.insert(elem.clone());
        }
    }

    for elem in both.iter() {
        first.remove(elem);
        second.remove(elem);
    }

    both
}

fn add_hash(file: &Path, to_root: &Path, manifest: &mut DeltaManifest) {
    let full = to_root.join(file);
    let mut md5hash = md5::Md5::new();
    println!("{:?}", full);
    let _ignore = std::io::copy(&mut std::fs::File::open(full).unwrap(), &mut md5hash);
    let res = md5hash.finalize();
    manifest.hashes.push(format!("{}:{}", hex::encode(res), file.to_str().unwrap()));
}

pub fn run_diff(from_root: &Path, to_root: &Path, patch_root: &Path, 
    src_only_files: HashSet<PathBuf>, dst_only_files: HashSet<PathBuf>, in_both_files: HashSet<PathBuf>,
    manifest: &mut DeltaManifest) {

    for path in src_only_files {
        println!("Deleting {}", path.to_str().unwrap());
        manifest.delete.push(path.to_str().unwrap().to_string());
    }
    
    for path in dst_only_files {
        println!("Adding new {}", path.to_str().unwrap());
        let mut src = std::fs::File::open(to_root.join(&path)).unwrap();
        let mut newpath = path.to_path_buf();
        let dst_path = patch_root.join(&newpath);
        match dst_path.parent() {
            None => {},
            Some(s) => std::fs::create_dir_all(s).unwrap()
        };
        let mut dst = std::fs::File::create(&dst_path).unwrap();
        std::io::copy(&mut src, &mut dst).unwrap();
        add_hash(&newpath, patch_root, manifest);
    }

    for path in in_both_files {
        println!("Diffing {}", path.to_str().unwrap());
        let mut newpath = path.to_path_buf();
        match path.extension().map(|x| x.to_str().unwrap()) {
            Some(exists) => {
                newpath.set_extension(format!("{}.patch", exists));
            },
            None => {
                newpath.set_extension("patch".to_owned());
            }
        }
        let dst_path = patch_root.join(&newpath);
        match dst_path.parent() {
            None => {},
            Some(s) => std::fs::create_dir_all(s).unwrap()
        };
        
        patch_file(&from_root.join(&path), &to_root.join(&path), &dst_path);
        add_hash(&path, to_root, manifest);
    }

    let manifest = serde_json::to_string(manifest).unwrap();
    std::fs::write(patch_root.join("manifest.json"), manifest.as_bytes()).unwrap();
}

pub fn tool(mut args: std::env::Args) {
    let from = args.next().unwrap_or("--help".to_owned());
    if from == "--help" || from == "-h" {
        dump_usage();
        return;
    }
    let to = args.next().unwrap_or("--help".to_owned());
    if to == "--help" || to == "-h" {
        dump_usage();
        return;
    }

    let src_path = Path::new(&from);
    let dst_path = Path::new(&to);
    if !fs::metadata(src_path).unwrap().is_dir() || !fs::metadata(dst_path).unwrap().is_dir() {
        println!("Error: Both source and dest must be directories");
        return;
    }

    let src_manifest : BuildManifest = serde_json::from_slice(&std::fs::read(src_path.join("manifest.json")).unwrap()).unwrap();
    let dst_manifest : BuildManifest = serde_json::from_slice(&std::fs::read(dst_path.join("manifest.json")).unwrap()).unwrap();

    let src_version = Version::create(src_manifest.version[0], src_manifest.version[1], src_manifest.version[2], src_manifest.dev);
    let dst_version = Version::create(dst_manifest.version[0], dst_manifest.version[1], dst_manifest.version[2], dst_manifest.dev);

    let patch = Patch::new(src_version, dst_version);
    println!("Assembling patch {}", String::from(&patch));

    let patch_dir = args.next().unwrap_or(format!("delta-{}", String::from(&patch)));
    let patch_dir = Path::new(&patch_dir);
    fs::create_dir_all(&patch_dir).unwrap();

    let mut from_files = get_all_files_recursive(src_path);
    let mut to_files = get_all_files_recursive(dst_path);
    let both_files = extract_overlap(&mut from_files, &mut to_files);


    let mut manifest = DeltaManifest {
        hashes: Vec::new(),
        delete: Vec::new(),
        target: dst_version,
        source: src_version,
        newExec: dst_manifest.exec
    };
    run_diff(src_path, dst_path, patch_dir, from_files, to_files, both_files, &mut manifest);
}