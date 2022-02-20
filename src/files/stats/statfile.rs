use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use fnv::FnvHashMap;
use std::convert::TryFrom;
use std::io::{Cursor, Read, Write};
use serde::ser::{Serializer, SerializeMap};

pub const STATFILE_MAGIC_NUMBER: u32 = 0x1EF1A757; // 57A7F11E "statfile"
const CURRENT_VERSION: u32 = 2;

pub const HEALTH_FLAG: u8 = 0;
pub const MASS_FLAG: u8 = 1;
pub const COST_FLAG: u8 = 2;
pub const PREMIUM_COST_FLAG: u8 = 8;
pub const RANKING_FLAG: u8 = 3;
pub const COMPLEXITY_FLAG: u8 = 4;
pub const THRUST_FLAG: u8 = 5;
pub const ROTATION_FLAG: u8 = 6;
pub const DAMAGE_FLAG: u8 = 7;
pub const SHIELD_FLAG: u8 = 9; // shield HP
pub const SHIELD_CHARGE_RATE_FLAG: u8 = 10; // rate of hp/sec
pub const SHIELD_CHARGE_DELAY_FLAG: u8 = 11; // millis after damage before start healing again
pub const USABILITY_HEALTH: u8 = 12; // How much "usable" HP there is (e.g. tesla blade charges)
pub const LIFT_FLAG: u8 = 13;

pub const SPECIAL_FLAG_1: u8 = 200; // Special per-part usage 0
pub const SPECIAL_FLAG_2: u8 = 201; // Special per-part usage 1
pub const SPECIAL_FLAG_3: u8 = 202; // Special per-part usage 2
#[derive(Clone, Serialize)]
pub struct StatsFile {
    pub blocks: FlagStats,
    pub attacks: FlagStats,
}

#[derive(Clone)]
pub struct FlagStats {
    pub data: FnvHashMap<u32, FnvHashMap<u8, i32>>,
}

impl Serialize for FlagStats {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut m2 = std::collections::BTreeMap::new();
        for (k, v) in &self.data {
            let mut mm = HashMap::new();
            for (k2, v2) in v {
                mm.insert(flag_name(*k2), v2);
            }
            m2.insert(k, mm);
        }

        let mut map = serializer.serialize_map(Some(m2.len()))?;
        for (k, v) in &m2 {  
            map.serialize_entry(&k.to_string(), &v)?;
        }
        map.end()
    }
}

impl FlagStats {
    fn new() -> FlagStats {
        FlagStats {
            data: FnvHashMap::default(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct JsonStatsFile {
    #[serde(rename = "blocks")]
    pub blocks: Vec<JsonBlockStats>,
    #[serde(rename = "attacks")]
    pub attacks: Vec<JsonAttackStats>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct JsonBlockStats {
    pub id: u32,
    pub name: String,
    #[serde(flatten)]
    pub flags: HashMap<String, i32>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct JsonAttackStats {
    pub id: u32,
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
        "shield" => Some(SHIELD_FLAG),
        "shieldCharge" => Some(SHIELD_CHARGE_RATE_FLAG),
        "shieldChargeDelay" => Some(SHIELD_CHARGE_DELAY_FLAG),
        "premiumCost" => Some(PREMIUM_COST_FLAG),
        "functionHealth" => Some(USABILITY_HEALTH),
        "damage" => Some(DAMAGE_FLAG),
        "lift" => Some(LIFT_FLAG),
        x => {
            if x.starts_with("spec1") {
                Some(SPECIAL_FLAG_1)
            } else if x.starts_with("spec2") {
                Some(SPECIAL_FLAG_2)
            } else if x.starts_with("spec3") {
                Some(SPECIAL_FLAG_3)
            } else {
                None
            }
        }
    }
}

fn flag_name(flag: u8) -> &'static str {
    match flag {
        HEALTH_FLAG =>"health",
        MASS_FLAG => "mass",
        COST_FLAG => "cost",
        RANKING_FLAG => "roboRanking",
        COMPLEXITY_FLAG => "cpuCost",
        THRUST_FLAG => "thrust",
        ROTATION_FLAG => "rotationSpeed",
        SHIELD_FLAG => "shield",
        DAMAGE_FLAG => "damage",
        SHIELD_CHARGE_RATE_FLAG => "shieldCharge",
        SHIELD_CHARGE_DELAY_FLAG => "shieldChargeDelay",
        PREMIUM_COST_FLAG => "premiumCost",
        USABILITY_HEALTH => "functionHealth",
        LIFT_FLAG => "lift",
        SPECIAL_FLAG_1 => "spec1",
        SPECIAL_FLAG_2 => "spec2",
        SPECIAL_FLAG_3 => "spec3",
        _ => "err"
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
        if magic != STATFILE_MAGIC_NUMBER {
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
            let mut map = FnvHashMap::default();
            elem.flags.iter().for_each(|flag| {
                let idn = flag_id(flag.0);
                if let Some(x) = idn {
                    map.insert(x, *flag.1);
                }
            });
            sf.blocks.data.insert(elem.id, map);
        });
        file.attacks.iter().for_each(|elem| {
            let mut map = FnvHashMap::default();
            elem.flags.iter().for_each(|flag| {
                let idn = flag_id(flag.0);
                if let Some(x) = idn {
                    map.insert(x, *flag.1);
                }
            });
            sf.attacks.data.insert(elem.id, map);
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
            let mut map = FnvHashMap::default();
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
            let mut map = FnvHashMap::default();
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
            let mut map = FnvHashMap::default();
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
        file.write_all(&u32::to_be_bytes(STATFILE_MAGIC_NUMBER))?; // "57A7F11E" STATFILE magic number
        file.write_all(&u32::to_be_bytes(CURRENT_VERSION))?;

        StatsFile::compile_sub(&self.blocks, &mut file)?;
        StatsFile::compile_sub(&self.attacks, &mut file)?;
        Ok(file.into_inner())
    }
}
