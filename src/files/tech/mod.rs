use std::io::{Cursor, Read, Write};

use serde::{Deserialize, Serialize};

pub const TECHTREE_MAGIC_NUMBER: u32 = 0x2ECC2AEE; // "TECCTREE"
pub const CURRENT_VERSION: u32 = 1;
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct AwardItem {
    pub item: i32,
    pub count: u32
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Rewards {
    #[serde(default)]
    pub part_unlock: Vec<i32>,
    #[serde(default)]
    pub cosmetic_unlock: Vec<i32>,
    #[serde(default)]
    pub part_award: Vec<AwardItem>,
    #[serde(default)]
    pub cosmetic_award: Vec<AwardItem>,
    #[serde(default)]
    pub background_unlock: Vec<i32>,
    #[serde(default)]
    pub environment_unlock: Vec<i32>,
    #[serde(default)]
    pub garage_slots: u8,
    #[serde(default)]
    pub prefab_bots: Vec<i32>
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TechItem {
    pub id: i64,
    pub name: String,
    pub cost: u64,
    pub prerequisite_tech: Vec<i64>,
    pub reward: Rewards
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TechTree {
    pub nodes: Vec<TechItem>
}

impl TechTree {
    fn compile_reward(file: &mut Cursor<Vec<u8>>, reward: &Rewards)-> Result<(), std::io::Error> {
        file.write_all(&u8::to_be_bytes(reward.part_unlock.len() as u8))?;
        for p in &reward.part_unlock {
            file.write_all(&i32::to_be_bytes(*p))?;
        }

        file.write_all(&u8::to_be_bytes(reward.cosmetic_unlock.len() as u8))?;
        for p in &reward.cosmetic_unlock {
            file.write_all(&i32::to_be_bytes(*p))?;
        }

        file.write_all(&u8::to_be_bytes(reward.part_award.len() as u8))?;
        for p in &reward.part_award {
            file.write_all(&i32::to_be_bytes(p.item))?;
            file.write_all(&u32::to_be_bytes(p.count))?;
        }

        file.write_all(&u8::to_be_bytes(reward.cosmetic_award.len() as u8))?;
        for p in &reward.cosmetic_award {
            file.write_all(&i32::to_be_bytes(p.item))?;
            file.write_all(&u32::to_be_bytes(p.count))?;
        }

        file.write_all(&u8::to_be_bytes(reward.background_unlock.len() as u8))?;
        for p in &reward.background_unlock {
            file.write_all(&i32::to_be_bytes(*p))?;
        }

        file.write_all(&u8::to_be_bytes(reward.environment_unlock.len() as u8))?;
        for p in &reward.environment_unlock {
            file.write_all(&i32::to_be_bytes(*p))?;
        }

        file.write_all(&u8::to_be_bytes(reward.garage_slots))?;

        file.write_all(&u8::to_be_bytes(reward.prefab_bots.len() as u8))?;

        for p in &reward.prefab_bots {
            file.write_all(&i32::to_be_bytes(*p))?;
        }
    
        Ok(())
    }

    pub fn compile(self: &TechTree) -> Result<Vec<u8>, std::io::Error> {
        let mut file = Cursor::new(Vec::new());
        file.write_all(&u32::to_be_bytes(TECHTREE_MAGIC_NUMBER))?;
        file.write_all(&u32::to_be_bytes(CURRENT_VERSION))?;
        file.write_all(&u64::to_be_bytes(self.nodes.len() as u64))?;

        for elem in self.nodes.iter() {
            file.write_all(&i64::to_be_bytes(elem.id))?;
            file.write_all(&u64::to_be_bytes(elem.cost))?;
            let n = elem.prerequisite_tech.len() as u32;
            file.write_all(&u32::to_be_bytes(n))?;
            for t in elem.prerequisite_tech.iter() {
                file.write_all(&i64::to_be_bytes(*t))?;
            }

            TechTree::compile_reward(&mut file, &elem.reward)?;
        }

        Ok(file.into_inner())
    }

    fn read_veci32(file: &mut Cursor<&[u8]>) -> Result<Vec<i32>, std::io::Error> {
        let mut buf1 = [0u8; 1];
        let mut buf4: [u8; 4] = [0u8; 4];
        file.read_exact(&mut buf1)?;
        (0..u8::from_be_bytes(buf1)).into_iter().map(|_| {
            file.read_exact(&mut buf4).map(|_| i32::from_be_bytes(buf4))
        }).collect::<std::io::Result<Vec<i32>>>()
    }

    fn read_vecaward(file: &mut Cursor<&[u8]>) -> Result<Vec<AwardItem>, std::io::Error> {
        let mut buf1 = [0u8; 1];
        let mut buf4: [u8; 4] = [0u8; 4];
        file.read_exact(&mut buf1)?;
        (0..u8::from_be_bytes(buf1)).into_iter().map(|_| {
            file.read_exact(&mut buf4)
                .map(|_| i32::from_be_bytes(buf4))
                .and_then(|x| { file.read_exact(&mut buf4).map(|_| x)})
                .map(|id| AwardItem { item: id, count: u32::from_be_bytes(buf4)})
        }).collect::<std::io::Result<Vec<AwardItem>>>()
    }

    fn from_v1(file: &mut Cursor<&[u8]>) -> Result<TechTree, std::io::Error> {
        let mut buf8 = [0u8; 8];
        let mut buf4 = [0u8; 4];
        let mut buf1 = [0u8; 1];

        file.read_exact(&mut buf8)?;
        let len = u64::from_be_bytes(buf8);

        let mut nodes= Vec::new();

        for _ in 0..len {
            file.read_exact(&mut buf8)?;
            let id = i64::from_be_bytes(buf8);

            file.read_exact(&mut buf8)?;
            let cost = u64::from_be_bytes(buf8);

            file.read_exact(&mut buf4)?;
            let num_prereqs= u32::from_be_bytes(buf4);
            let prereqs = (0..num_prereqs).into_iter().map(|x| {
                file.read_exact(&mut buf8).map(|_| i64::from_be_bytes(buf8))
            }).collect::<std::io::Result<Vec<i64>>>()?;

            let part_unlock = TechTree::read_veci32(file)?;
            let cosmetic_unlock = TechTree::read_veci32(file)?;
            let part_award = TechTree::read_vecaward(file)?;
            let cosmetic_award = TechTree::read_vecaward(file)?;
            let background_unlock = TechTree::read_veci32(file)?;
            let environment_unlock = TechTree::read_veci32(file)?;

            file.read_exact(&mut buf1)?;
            let garage_slots = u8::from_be_bytes(buf1);
            let prefab_bots = TechTree::read_veci32(file)?;

            let reward = Rewards {
                part_unlock,
                cosmetic_unlock,
                part_award,
                cosmetic_award,
                background_unlock,
                environment_unlock,
                garage_slots,
                prefab_bots,
            };
            nodes.push(TechItem { id, name: "".to_owned(), cost, prerequisite_tech: prereqs, reward });
        }

        Ok(TechTree { nodes })
    }
}

impl TryFrom<&[u8]> for TechTree {
    type Error = std::io::Error;
    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        let mut file = Cursor::new(data);
        let mut buf4 = [0u8; 4];
        file.read_exact(&mut buf4)?;
        let magic = u32::from_be_bytes(buf4);
        if magic != TECHTREE_MAGIC_NUMBER {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Magic number was invalid: {magic}"),
            ));
        }

        file.read_exact(&mut buf4)?;
        let version = u32::from_be_bytes(buf4);
        let res: Result<TechTree, std::io::Error> = match version {
            1 => TechTree::from_v1(&mut file),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Version was invalid: {version}"),
            )),
        };
        res
    }
}

