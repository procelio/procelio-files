use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::io::{Cursor, Read, Write};
use fnv;
pub const INVENTORY_MAGIC_NUMBER: u32 = 0xC50CB115; // 15B10CC5 "IsBloccs"
const CURRENT_VERSION: u32 = 2;


#[derive(Clone, Serialize, Deserialize)]
pub struct JsonInventory {
    parts: Vec<JsonPartCount>
}

#[derive(Clone, Serialize, Deserialize)]
pub struct JsonPartCount {
    pub id: u32,
    pub name: String,
    pub count: u32
}

#[derive(Clone, Serialize)]
pub struct PartCount {
    pub id: u32,
    pub count: u32
}

#[derive(Serialize, Deserialize)]
pub struct Inventory {
    parts: fnv::FnvHashMap<u32, u32>
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
                format!("Magic number was invalid: {}", magic),
            ));
        }

        file.read_exact(&mut buf4)?;
        let version = u32::from_be_bytes(buf4);
        let res: Result<(), std::io::Error> = match version {
            1 => Inventory::from_v1(&mut blank, &mut file),
            2 => Inventory::from_v2(&mut blank, &mut file),
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

impl From<JsonInventory> for Inventory {
    fn from(file: JsonInventory) -> Self {
        let mut sf = Inventory::new();
        file.parts.iter().for_each(|elem| {
            sf.parts.insert(elem.id, elem.count);
        });
        sf
    }
}

impl Inventory {
    // Add "from"'s data into "into", returning Ok(Inv) if successful or Err(msg) if not
    // Will fail if u32 overflow occurs
    #[allow(dead_code)] // lib function
    pub fn add_inventories(from: Inventory, mut into: Inventory) -> Result<Inventory, String> {
        for elem in from.parts {
            let summed = into.parts.get(&elem.0).unwrap_or(&0u32).checked_add(elem.1);
            match summed {
                None => { return Err(format!("u32 overflow occurred for part {}", elem.0)); },
                Some(s) => { into.parts.insert(elem.0, s); }
            }
        }
        Ok(into)
    }

    // Subtract "from"'s data from "into"
    // Returns Ok(Inv) if successful, Err(msg) if not (e.g. count would drop below zero)
    #[allow(dead_code)] // lib function
    pub fn subtract_inventories(from: Inventory, mut into: Inventory) -> Result<Inventory, String> {
        for elem in from.parts {
            let in_into = *into.parts.get(&elem.0).unwrap_or(&0u32);
            match elem.1 > in_into {
                true => { return Err(format!("Tried to remove too many of {}: count would drop below 0", elem.0)) },
                false => { into.parts.insert(elem.0, in_into - elem.1); }
            }
        }
        Ok(into)
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
            inv.parts.insert(id.into(), val);
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
            let part_count = u32::from_be_bytes(buf4);
            inv.parts.insert(part_id, part_count);
        }
        Ok(())
    }

    pub fn new() -> Inventory {
        Inventory {
            parts: fnv::FnvHashMap::with_capacity_and_hasher(20, Default::default())
        }
    }

    pub fn compile(self: &Inventory) -> Result<Vec<u8>, std::io::Error> {
        let mut file = Cursor::new(Vec::new());
        file.write_all(&u32::to_be_bytes(INVENTORY_MAGIC_NUMBER))?; // "57A7F11E" STATFILE magic number
        file.write_all(&u32::to_be_bytes(CURRENT_VERSION))?;
        file.write_all(&u32::to_be_bytes(self.parts.len() as u32))?;
        for elem in self.parts.iter() {
            file.write_all(&u32::to_be_bytes(*elem.0))?;
            file.write_all(&u32::to_be_bytes(*elem.1))?;
        }
        Ok(file.into_inner())
    }
}
