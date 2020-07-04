use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::io::{Cursor, Read, Write};

static MAGIC_NUMBER: u32 = 0x1EF1A757;
static CURRENT_VERSION: u32 = 2;

pub static HEALTH_FLAG: u8 = 0;
pub static MASS_FLAG: u8 = 1;
pub static COST_FLAG: u8 = 2;
pub static RANKING_FLAG: u8 = 3;
pub static COMPLEXITY_FLAG: u8 = 4;
pub static THRUST_FLAG: u8 = 5;
pub static ROTATION_FLAG: u8 = 6;
pub static DAMAGE_FLAG: u8 = 7;

#[derive(Clone)]
pub struct StatsFile {
    pub blocks: FlagStats,
    pub attacks: FlagStats,
}

#[derive(Clone)]
pub struct FlagStats {
    pub data: HashMap<u32, HashMap<u8, i32>>,
}

impl FlagStats {
    fn new() -> FlagStats {
        FlagStats {
            data: HashMap::new(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct JsonStatsFile {
    #[serde(rename = "blocks")]
    pub blocks: HashMap<u32, JsonBlockStats>,
    #[serde(rename = "attacks")]
    pub attacks: HashMap<u32, JsonAttackStats>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct JsonBlockStats {
    pub name: String,
    #[serde(flatten)]
    pub flags: HashMap<String, i32>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct JsonAttackStats {
    pub name: String,
    #[serde(flatten)]
    pub flags: HashMap<String, i32>,
}

fn flag_id(flag: &str) -> Option<u8> {
    match flag {
        "health" => Some(HEALTH_FLAG),
        "mass" => Some(MASS_FLAG),
        "cost" => Some(COST_FLAG),
        "roboRanking" => Some(RANKING_FLAG),
        "cpuCost" => Some(COMPLEXITY_FLAG),
        "thrust" => Some(THRUST_FLAG),
        "rotationSpeed" => Some(ROTATION_FLAG),

        "damage" => Some(DAMAGE_FLAG),
        _ => None
    }
}

impl TryFrom<&[u8]> for StatsFile {
    type Error = std::io::Error;
    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        let mut blank = StatsFile::new();
        let mut file = Cursor::new(data);
        let mut buf4 = [0u8; 4];
        file.read_exact(&mut buf4)?;
        let magic = u32::from_be_bytes(buf4);
        if magic != MAGIC_NUMBER {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Magic number was invalid: {}", magic),
            ));
        }

        file.read_exact(&mut buf4)?;
        let version = u32::from_be_bytes(buf4);
        let res: Result<(), std::io::Error> = match version {
            1 => StatsFile::from_v1(&mut blank, &mut file),
            2 => StatsFile::from_v2(&mut blank, &mut file),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Version was invalid: {}", version),
            )),
        };
        if let Err(e) = res {
            return Err(e);
        }

        Ok(blank)
    }
}

impl From<JsonStatsFile> for StatsFile {
    fn from(file: JsonStatsFile) -> Self {
        let mut sf = StatsFile::new();
        file.blocks.iter().for_each(|elem| {
            let mut map = HashMap::new();
            elem.1.flags.iter().for_each(|flag| {
                let idn = flag_id(flag.0);
                if let Some(x) = idn {
                    map.insert(x, *flag.1);
                }
            });
            sf.blocks.data.insert(*elem.0, map);
        });
        file.attacks.iter().for_each(|elem| {
            let mut map = HashMap::new();
            elem.1.flags.iter().for_each(|flag| {
                let idn = flag_id(flag.0);
                if let Some(x) = idn {
                    map.insert(x, *flag.1);
                }
            });
            sf.attacks.data.insert(*elem.0, map);
        });
        sf
    }
}

impl StatsFile {
    fn from_v1(stats: &mut StatsFile, file: &mut Cursor<&[u8]>) -> Result<(), std::io::Error> {
        let mut buf4 = [0u8; 4];
        let mut buf2 = [0u8; 2];
        let mut buf1 = [0u8; 1];
        file.read_exact(&mut buf4)?;
        let ct = u32::from_be_bytes(buf4);

        for _ in 0..ct {
            file.read_exact(&mut buf2)?;
            let id = u16::from_be_bytes(buf2);
            file.read_exact(&mut buf1)?;
            let fc = u8::from_be_bytes(buf1);
            let mut map = HashMap::new();
            for _ in 0..fc {
                file.read_exact(&mut buf1)?;
                let flag = u8::from_be_bytes(buf1);
                file.read_exact(&mut buf4)?;
                let value = i32::from_be_bytes(buf4);
                map.insert(flag, value);
            }
            stats.blocks.data.insert(id.into(), map);
        }

        Ok(())
    }

    fn from_v2(stats: &mut StatsFile, file: &mut Cursor<&[u8]>) -> Result<(), std::io::Error> {
        let mut buf4 = [0u8; 4];
        let mut buf1 = [0u8; 1];

        file.read_exact(&mut buf4)?;
        let num_blocks = u32::from_be_bytes(buf4);
        for _ in 0..num_blocks {
            file.read_exact(&mut buf4)?;
            let entity_id = u32::from_be_bytes(buf4);
            file.read_exact(&mut buf1)?;
            let num_flags = u8::from_be_bytes(buf1);
            let mut map = HashMap::new();
            for _ in 0..num_flags {
                file.read_exact(&mut buf1)?;
                let flag = u8::from_be_bytes(buf1);
                file.read_exact(&mut buf4)?;
                let value = i32::from_be_bytes(buf4);
                map.insert(flag, value);
            }
            stats.blocks.data.insert(entity_id, map);
        }

        file.read_exact(&mut buf4)?;
        let num_attacks = u32::from_be_bytes(buf4);
        for _ in 0..num_attacks {
            file.read_exact(&mut buf4)?;
            let entity_id = u32::from_be_bytes(buf4);
            file.read_exact(&mut buf1)?;
            let num_flags = u8::from_be_bytes(buf1);
            let mut map = HashMap::new();
            for _ in 0..num_flags {
                file.read_exact(&mut buf1)?;
                let flag = u8::from_be_bytes(buf1);
                file.read_exact(&mut buf4)?;
                let value = i32::from_be_bytes(buf4);
                map.insert(flag, value);
            }
            stats.attacks.data.insert(entity_id, map);
        }

        Ok(())
    }

    pub fn new() -> StatsFile {
        StatsFile {
            blocks: FlagStats::new(),
            attacks: FlagStats::new(),
        }
    }

    fn compile_sub(stat: &FlagStats, file: &mut Cursor<Vec<u8>>) -> Result<(), std::io::Error> {
        file.write_all(&u32::to_be_bytes(stat.data.len() as u32))?;
        for kvp in &stat.data {
            file.write_all(&u32::to_be_bytes(*kvp.0))?;
            file.write_all(&u8::to_be_bytes(kvp.1.len() as u8))?;
            for kvp2 in kvp.1 {
                file.write_all(&u8::to_be_bytes(*kvp2.0))?;
                file.write_all(&i32::to_be_bytes(*kvp2.1))?;
            }
        }

        Ok(())
    }

    pub fn compile(self: &StatsFile) -> Result<Vec<u8>, std::io::Error> {
        let mut file = Cursor::new(Vec::new());
        file.write_all(&u32::to_be_bytes(MAGIC_NUMBER))?; // "57A7F11E" STATFILE magic number
        file.write_all(&u32::to_be_bytes(CURRENT_VERSION))?;

        StatsFile::compile_sub(&self.blocks, &mut file)?;
        StatsFile::compile_sub(&self.attacks, &mut file)?;
        Ok(file.into_inner())
    }
}
