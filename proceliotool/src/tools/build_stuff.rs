use std::cmp::{PartialEq, Eq, Ord, Ordering};
use serde::{Serialize, Deserialize};
use std::convert::TryFrom;
use std::hash::*;
use regex::Regex;

#[derive(Serialize, Deserialize)]
pub struct BuildManifest {
    pub version: String,
    pub channel: String,
    pub exec: String
}

#[derive(Clone, Serialize, Deserialize, Hash, Debug)]
pub struct Version {
    pub version: String,
    pub channel: String
}

impl Version {
    pub fn new(version: String, channel: String) -> Self {
        Self {
            version, channel
        }
    }
    pub fn create(major: u32, minor: u32, patch: u32, channel: String) -> Version {
        Version::new(format!("{major}.{minor}.{patch}"), channel)
    }
}
impl From<&Version> for String {
    fn from(v: &Version) -> String {
        format!("{}-{}", v.version, v.channel)
    }
}

#[derive(Clone, Serialize, Deserialize, Hash, Debug)]
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
        self.version == other.version && self.channel == other.channel
    }
}
impl Eq for Version {}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        self.version.cmp(&other.version).then(self.channel.cmp(&other.channel))
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
    static ref PATCH: Regex = Regex::new(r"^([^_]*)(_?([^_]*))-([^_]*)(_?([^_]*))$").unwrap();
    static ref VERSION: Regex = Regex::new(r"^([^_]*)(_?([^_]*))$").unwrap();
}

impl TryFrom<&str> for Version {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let version_cap = VERSION.captures(s);
        if let Some(cap) = version_cap {
            let v1 = cap.get(1).unwrap().as_str().to_owned();
            let v2 = cap.get(3).map(|x| x.as_str().to_owned());
            Ok(Version { version: v1, channel: v2.unwrap_or("prod".to_owned())})
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
            let v1 = cap.get(1).unwrap().as_str().to_owned();
            let v2 = cap.get(3).map(|x| x.as_str().to_owned());
          
            let v4 = cap.get(4).unwrap().as_str().to_owned();
            let v6 = cap.get(6).map(|x| x.as_str().to_owned());

            Ok(Patch::new(
                Version::new(v1, v2.unwrap_or("prod".to_owned())),
                Version::new(v4, v6.unwrap_or("prod".to_owned())),
            ))
        } else {
            Err(format!("unable to parse patch {}", s))
        }
    }
}