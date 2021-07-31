use std::cmp::{PartialEq, Eq, Ord, Ordering};
use serde::{Serialize, Deserialize};
use std::convert::TryFrom;
use std::hash::*;
use regex::Regex;

#[derive(Serialize, Deserialize)]
pub struct BuildManifest {
    pub version: Vec<u32>,
    pub dev: bool,
    pub exec: String
}

#[derive(Clone, Copy, Serialize, Deserialize, Hash, Debug)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub dev_build: bool
}

impl Version {
    pub fn new() -> Version {
        Version {
            major: 0,
            minor: 0,
            patch: 0,
            dev_build: false
        }
    }
    pub fn create(major: u32, minor: u32, patch: u32, dev: bool) -> Version {
        Version {
            major: major,
            minor: minor,
            patch: patch,
            dev_build: dev
        }
    }
}
impl From<&Version> for String {
    fn from(v: &Version) -> String {
        format!("{}.{}.{}{}", v.major, v.minor, v.patch, if v.dev_build { "dev"} else { "" })
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, Hash, Debug)]
pub struct Patch {
    pub from: Version,
    pub to: Version
}

impl Patch {
    pub fn new(v_from: Version, v_to: Version) -> Patch {
        Patch {
            from: v_from,
            to: v_to
        }
    }
}
impl From<&Patch> for String {
    fn from(p: &Patch) -> String {
        format!("{}-{}", String::from(&p.from), String::from(&p.to))
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.major == other.major && self.minor == other.minor 
        && self.patch == other.patch && self.dev_build == other.dev_build
    }
}
impl Eq for Version {}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        self.major.cmp(&other.major).then(self.minor.cmp(&other.minor))
            .then(self.patch.cmp(&other.patch)).then(self.dev_build.cmp(&other.dev_build))
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}


impl PartialEq for Patch {
    fn eq(&self, other: &Self) -> bool {
        self.from == other.from && self.to == other.to
    }
}

impl Eq for Patch {}

impl Ord for Patch {
    fn cmp(&self, other: &Self) -> Ordering {
        self.from.cmp(&other.from).then(self.to.cmp(&other.to))
    }
}

impl PartialOrd for Patch {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

use lazy_static::lazy_static;

lazy_static! {
    static ref PATCH: Regex = Regex::new(r"(?:(?:[^\d]*)|(?:.*[^\d]))(\d+)\.(\d+)\.(\d+)(dev)?-(\d+)\.(\d+)\.(\d+)(dev)?.*").unwrap();
    static ref VERSION: Regex = Regex::new(r"(?:(?:[^\d]*)|(?:.*[^\d]))(\d+)\.(\d+)\.(\d+)(dev)?").unwrap();
}
impl TryFrom<&str> for Version {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let version_cap = VERSION.captures(s);
        if let Some(cap) = version_cap {
            let v1: u32 = cap.get(1).unwrap().as_str().parse().unwrap();
            let v2: u32 = cap.get(2).unwrap().as_str().parse().unwrap();
            let v3: u32 = cap.get(3).unwrap().as_str().parse().unwrap();
            let dev = cap.get(4).is_some();
            Ok(Version {major: v1, minor: v2, patch: v3, dev_build: dev})
        } else {
            Err(format!("unable to parse version {}", s))
        }
    }
}
impl TryFrom<&str> for Patch {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let patch_cap = PATCH.captures(s);
        if let Some(cap) = patch_cap {
            let v11: u32 = cap.get(1).unwrap().as_str().parse().unwrap();
            let v12: u32 = cap.get(2).unwrap().as_str().parse().unwrap();
            let v13: u32 = cap.get(3).unwrap().as_str().parse().unwrap();
            let first_dev = cap.get(4).is_some();
          
            let v21: u32 = cap.get(5).unwrap().as_str().parse().unwrap();
            let v22: u32 = cap.get(6).unwrap().as_str().parse().unwrap();
            let v23: u32 = cap.get(7).unwrap().as_str().parse().unwrap();
            let second_dev = cap.get(8).is_some();


            Ok(Patch::new(
                Version {major: v11, minor: v12, patch: v13, dev_build: first_dev},
                Version {major: v21, minor: v22, patch: v23, dev_build: second_dev}
            ))
        } else {
            Err(format!("unable to parse patch {}", s))
        }
    }
}