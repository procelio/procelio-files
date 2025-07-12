use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::io::{Cursor, Read, Write};
use fnv;
use crate::files::robot::robot::Robot;
pub const INVENTORY_MAGIC_NUMBER: u32 = 0xC50CB115; // 15B10CC5 "IsBloccs"
const CURRENT_VERSION: u32 = 3;

#[derive(Clone, Serialize, Deserialize)]
pub struct JsonInventory {
    parts: Vec<JsonPartCount>,
    cosmetics: Vec<JsonPartCount>
}

#[derive(Clone, Serialize, Deserialize)]
pub struct JsonPartCount {
    pub id: u32,
    pub name: String,
    pub count: i32
}

#[derive(Serialize, Deserialize)]
pub struct Inventory {
    pub parts: fnv::FnvHashMap<u32, i32>,
    pub cosmetics: fnv::FnvHashMap<u32, i32>
}

impl TryFrom<&[u8]> for Inventory {
    type Error = std::io::Error;
    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        let mut blank = Inventory::new();
        let mut file = Cursor::new(data);
        let mut buf4 = [0u8; 4];
        file.read_exact(&mut buf4)?;
        let magic = u32::from_be_bytes(buf4);
        if magic != INVENTORY_MAGIC_NUMBER {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Magic number was invalid: {magic}"),
            ));
        }

        file.read_exact(&mut buf4)?;
        let version = u32::from_be_bytes(buf4);
        let res: Result<(), std::io::Error> = match version {
            1 => Inventory::from_v1(&mut blank, &mut file),
            2 => Inventory::from_v2(&mut blank, &mut file),
            3 => Inventory::from_v3(&mut blank, &mut file),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Version was invalid: {version}"),
            )),
        };
        res?;

        Ok(blank)
    }
}

impl From<JsonInventory> for Inventory {
    fn from(file: JsonInventory) -> Self {
        let mut sf = Inventory::new();
        file.parts.iter().for_each(|elem| {
            sf.parts.insert(elem.id, elem.count);
        });
        file.cosmetics.iter().for_each(|elem| {
            sf.cosmetics.insert(elem.id, elem.count);
        });
        sf
    }
}

impl From<Robot> for Inventory {
    fn from(bot: Robot) -> Self {
        let mut inv = Inventory::new();
        bot.parts.iter().for_each(|elem| {
            inv.add_part(elem.id, 1);
        });
        bot.cosmetics.iter().for_each(|elem| {
            inv.add_cosmetic(elem.id, 1);
        });
        inv
    }
}

impl From<Inventory> for JsonInventory {
    fn from(inv: Inventory) -> Self {
        JsonInventory {
            parts: inv.parts.into_iter().map(|x| JsonPartCount {
                id: x.0,
                count: x.1,
                name: "?".to_owned()
            }).collect(),
            cosmetics: inv.cosmetics.into_iter().map(|x| JsonPartCount {
                id: x.0,
                count: x.1,
                name: "?".to_owned()
            }).collect()
        }
    }
}

impl Inventory {
    // Add "from"'s data into "into", returning Ok(Inv) if successful or Err(msg) if not
    // Will fail if u32 overflow occurs
    #[allow(dead_code)] // lib function
    pub fn add_inventories(from: &Inventory, mut into: Inventory) -> Result<Inventory, String> {
        for elem in from.parts.iter() {
            let summed = into.parts.get(elem.0).unwrap_or(&0i32).checked_add(*elem.1);
            match summed {
                None => { return Err(format!("u32 overflow occurred for part {}", elem.0)); },
                Some(s) => { into.parts.insert(*elem.0, s); }
            }
        }

        for elem in from.cosmetics.iter() {
            let summed = into.cosmetics.get(elem.0).unwrap_or(&0i32).checked_add(*elem.1);
            match summed {
                None => { return Err(format!("u32 overflow occurred for cosmetic {}", elem.0)); },
                Some(s) => { into.cosmetics.insert(*elem.0, s); }
            }
        }

        Ok(into)
    }

    // Subtract "from"'s data from "into"
    // Returns Ok(Inv) if successful, Err(inv) if not (e.g. count would drop below zero)
    #[allow(dead_code)] // lib function
    pub fn subtract_inventories(from: &Inventory, mut into: Inventory) -> Result<Inventory, Inventory> {
        let mut negative = false;
        for elem in from.parts.iter() {
            let in_into = *into.parts.get(elem.0).unwrap_or(&0i32);
            if *elem.1 > in_into {
                negative = true;
            }
            into.parts.insert(*elem.0, in_into - elem.1);
        }
        for elem in from.cosmetics.iter() {
            let in_into = *into.cosmetics.get(elem.0).unwrap_or(&0i32);
            if *elem.1 > in_into {
                negative = true;
            }
            into.cosmetics.insert(*elem.0, in_into - elem.1);
        }
        if negative { Err(into) } else { Ok(into) }
    }

    fn from_v1(inv: &mut Inventory, file: &mut Cursor<&[u8]>) -> Result<(), std::io::Error> {
        let mut buf4 = [0u8; 4];
        let mut buf2 = [0u8; 2];
        file.read_exact(&mut buf4)?;
        let ct = u32::from_be_bytes(buf4);

        for _ in 0..ct {
            file.read_exact(&mut buf2)?;
            let id = u16::from_be_bytes(buf2);
            file.read_exact(&mut buf4)?;
            let val = u32::from_be_bytes(buf4);
            inv.parts.insert(id.into(), val as i32);
        }

        Ok(())
    }

    fn from_v2(inv: &mut Inventory, file: &mut Cursor<&[u8]>) -> Result<(), std::io::Error> {
        let mut buf4 = [0u8; 4];

        file.read_exact(&mut buf4)?;
        let num_elems = u32::from_be_bytes(buf4);
        for _ in 0..num_elems {
            file.read_exact(&mut buf4)?;
            let part_id = u32::from_be_bytes(buf4);
            file.read_exact(&mut buf4)?;
            let part_count = i32::from_be_bytes(buf4);
            inv.parts.insert(part_id, part_count);
        }
        Ok(())
    }

    fn from_v3(inv: &mut Inventory, file: &mut Cursor<&[u8]>) -> Result<(), std::io::Error> {
        let mut buf4 = [0u8; 4];

        Inventory::from_v2(inv, file)?;

        file.read_exact(&mut buf4)?;
        let num_elems = u32::from_be_bytes(buf4);
        for _ in 0..num_elems {
            file.read_exact(&mut buf4)?;
            let part_id = u32::from_be_bytes(buf4);
            file.read_exact(&mut buf4)?;
            let part_count = i32::from_be_bytes(buf4);
            inv.cosmetics.insert(part_id, part_count);
        }
        Ok(())
    }

    pub fn new() -> Inventory {
        Inventory {
            parts: fnv::FnvHashMap::with_capacity_and_hasher(20, Default::default()),
            cosmetics: fnv::FnvHashMap::default()
        }
    }

    pub fn compile(self: &Inventory) -> Result<Vec<u8>, std::io::Error> {
        let mut file = Cursor::new(Vec::new());
        file.write_all(&u32::to_be_bytes(INVENTORY_MAGIC_NUMBER))?; // "57A7F11E" STATFILE magic number
        file.write_all(&u32::to_be_bytes(CURRENT_VERSION))?;
        file.write_all(&u32::to_be_bytes(self.parts.len() as u32))?;
        for elem in self.parts.iter() {
            file.write_all(&u32::to_be_bytes(*elem.0))?;
            file.write_all(&i32::to_be_bytes(*elem.1))?;
        }

        file.write_all(&u32::to_be_bytes(self.cosmetics.len() as u32))?;
        for elem in self.cosmetics.iter() {
            file.write_all(&u32::to_be_bytes(*elem.0))?;
            file.write_all(&i32::to_be_bytes(*elem.1))?;
        }

        Ok(file.into_inner())
    }

    pub fn add_part(self: &mut Inventory, part: u32, count: i32) {
        self.parts.insert(part, self.parts.get(&part).unwrap_or(&0) + count);
    }

    pub fn add_cosmetic(self: &mut Inventory, cosmetic: u32, count: i32) {
        self.cosmetics.insert(cosmetic, self.cosmetics.get(&cosmetic).unwrap_or(&0) + count);
    }
}
