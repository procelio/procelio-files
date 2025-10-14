use std::io::{Cursor, Read, Write};

use serde::{Deserialize, Serialize};

pub const TECHTREE_MAGIC_NUMBER: u32 = 0x2ECC2AEE; // "TECCTREE"
pub const CURRENT_VERSION: u32 = 1;
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct AwardItem {
    pub item: u32,
    pub count: u32
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Rewards {
    #[serde(default)]
    pub part_unlock: Vec<u32>,
    #[serde(default)]
    pub cosmetic_unlock: Vec<u32>,
    #[serde(default)]
    pub part_award: Vec<AwardItem>,
    #[serde(default)]
    pub cosmetic_award: Vec<AwardItem>,
    #[serde(default)]
    pub background_unlock: Vec<u32>,
    #[serde(default)]
    pub environment_unlock: Vec<u32>,
    #[serde(default)]
    pub garage_slots: u8,
    #[serde(default)]
    pub currency_award: u32,
    #[serde(default)]
    pub premium_currency_award: u32,
    #[serde(default)]
    pub prefab_bots: Vec<u32>
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct TechID(pub i64);

impl TechID {
    pub fn new(id: i64) -> Self { Self(id) }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TechItem {
    pub id: TechID,
    pub name: String,
    pub cost: i64,
    pub prerequisite_tech: Vec<TechID>,
    pub prereqs_and: bool,
    pub reward: Rewards
}

impl TechItem {
    pub fn may_unlock(&self, ordered_player_tech: &[TechID]) -> bool {
        if self.prereqs_and {
            self.prerequisite_tech.iter().all(|x| ordered_player_tech.binary_search(&x).is_ok())
        } else {
            self.prerequisite_tech.iter().any(|x| ordered_player_tech.binary_search(&x).is_ok()) || self.prerequisite_tech.len() == 0
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TechTree {
    pub nodes: Vec<TechItem>
}

/*
{ "nodes": [
  { 
    "id": 1,
    "name": idk",
    "cost": 10000,
    "prerequisite_tech": [],
    "reward": { "currency_award": 30000 }
  }


]}

*/

impl TechTree {
    fn compile_reward(file: &mut Cursor<Vec<u8>>, reward: &Rewards)-> Result<(), std::io::Error> {
        file.write_all(&u8::to_be_bytes(reward.part_unlock.len() as u8))?;
        for p in &reward.part_unlock {
            file.write_all(&u32::to_be_bytes(*p))?;
        }

        file.write_all(&u8::to_be_bytes(reward.cosmetic_unlock.len() as u8))?;
        for p in &reward.cosmetic_unlock {
            file.write_all(&u32::to_be_bytes(*p))?;
        }

        file.write_all(&u8::to_be_bytes(reward.part_award.len() as u8))?;
        for p in &reward.part_award {
            file.write_all(&u32::to_be_bytes(p.item))?;
            file.write_all(&u32::to_be_bytes(p.count))?;
        }

        file.write_all(&u8::to_be_bytes(reward.cosmetic_award.len() as u8))?;
        for p in &reward.cosmetic_award {
            file.write_all(&u32::to_be_bytes(p.item))?;
            file.write_all(&u32::to_be_bytes(p.count))?;
        }

        file.write_all(&u8::to_be_bytes(reward.background_unlock.len() as u8))?;
        for p in &reward.background_unlock {
            file.write_all(&u32::to_be_bytes(*p))?;
        }

        file.write_all(&u8::to_be_bytes(reward.environment_unlock.len() as u8))?;
        for p in &reward.environment_unlock {
            file.write_all(&u32::to_be_bytes(*p))?;
        }

        file.write_all(&u8::to_be_bytes(reward.garage_slots))?;

        file.write_all(&u8::to_be_bytes(reward.prefab_bots.len() as u8))?;

        file.write_all(&u32::to_be_bytes(reward.currency_award))?;
        file.write_all(&u32::to_be_bytes(reward.premium_currency_award))?;

        for p in &reward.prefab_bots {
            file.write_all(&u32::to_be_bytes(*p))?;
        }
    
        Ok(())
    }

    pub fn compile(self: &TechTree) -> Result<Vec<u8>, std::io::Error> {
        let mut file = Cursor::new(Vec::new());
        file.write_all(&u32::to_be_bytes(TECHTREE_MAGIC_NUMBER))?;
        file.write_all(&u32::to_be_bytes(CURRENT_VERSION))?;
        file.write_all(&u64::to_be_bytes(self.nodes.len() as u64))?;

        for elem in self.nodes.iter() {
            file.write_all(&i64::to_be_bytes(elem.id.0))?;
            file.write_all(&i64::to_be_bytes(elem.cost))?;
            let n = elem.prerequisite_tech.len() as u32;
            file.write_all(&u32::to_be_bytes(n))?;
            for t in elem.prerequisite_tech.iter() {
                file.write_all(&i64::to_be_bytes(t.0))?;
            }
            file.write_all(&u8::to_be_bytes(if elem.prereqs_and { 1 } else { 0 }))?;

            TechTree::compile_reward(&mut file, &elem.reward)?;
        }

        Ok(file.into_inner())
    }

    fn read_vecu32(file: &mut Cursor<&[u8]>) -> Result<Vec<u32>, std::io::Error> {
        let mut buf1 = [0u8; 1];
        let mut buf4: [u8; 4] = [0u8; 4];
        file.read_exact(&mut buf1)?;
        (0..u8::from_be_bytes(buf1)).map(|_| {
            file.read_exact(&mut buf4).map(|_| u32::from_be_bytes(buf4))
        }).collect::<std::io::Result<Vec<u32>>>()
    }

    fn read_vecaward(file: &mut Cursor<&[u8]>) -> Result<Vec<AwardItem>, std::io::Error> {
        let mut buf1 = [0u8; 1];
        let mut buf4: [u8; 4] = [0u8; 4];
        file.read_exact(&mut buf1)?;
        (0..u8::from_be_bytes(buf1)).map(|_| {
            file.read_exact(&mut buf4)
                .map(|_| u32::from_be_bytes(buf4))
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
            let id = TechID(i64::from_be_bytes(buf8));

            file.read_exact(&mut buf8)?;
            let cost = i64::from_be_bytes(buf8);

            file.read_exact(&mut buf4)?;
            let num_prereqs= u32::from_be_bytes(buf4);
            let prereqs = (0..num_prereqs).map(|_| {
                file.read_exact(&mut buf8).map(|_| TechID(i64::from_be_bytes(buf8)))
            }).collect::<std::io::Result<Vec<TechID>>>()?;
            file.read_exact(&mut buf1)?;
            let prereqs_and = u8::from_be_bytes(buf1) > 0;

            let part_unlock = TechTree::read_vecu32(file)?;
            let cosmetic_unlock = TechTree::read_vecu32(file)?;
            let part_award = TechTree::read_vecaward(file)?;
            let cosmetic_award = TechTree::read_vecaward(file)?;
            let background_unlock = TechTree::read_vecu32(file)?;
            let environment_unlock = TechTree::read_vecu32(file)?;

            file.read_exact(&mut buf4)?;
            let currency_award = u32::from_be_bytes(buf4);

            file.read_exact(&mut buf4)?;
            let premium_currency_award = u32::from_be_bytes(buf4);

            file.read_exact(&mut buf1)?;
            let garage_slots = u8::from_be_bytes(buf1);
            let prefab_bots = TechTree::read_vecu32(file)?;

            let reward = Rewards {
                part_unlock,
                cosmetic_unlock,
                part_award,
                cosmetic_award,
                background_unlock,
                environment_unlock,
                garage_slots,
                currency_award,
                premium_currency_award,
                prefab_bots,
            };
            nodes.push(TechItem { id, name: "".to_owned(), cost, prerequisite_tech: prereqs, prereqs_and, reward });
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

