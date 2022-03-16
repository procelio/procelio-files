use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::io::{Cursor, Read, Write};
use std::io::Seek;
use md5::{Md5, Digest};

pub const ROBOT_MAGIC_NUMBER: u32 = 0xC571B040; // 40B071C5 "Robotics"
const CURRENT_VERSION: u32 = 3;
pub const MAX_EXTRADATA_SIZE: u8 = 64;
#[derive(Clone, Serialize, Deserialize)]
pub struct JsonRobot {
    name: String,
    metadata: u64,
    parts: Vec<JsonPart>,
    cosmetics: Vec<JsonCosmetic>
}

#[derive(Clone, Serialize, Deserialize)]
pub struct JsonPart {
    id: u32,
    pos: [i8; 3],
    rot: u8,
    color: [u8; 3],
    alpha: u8,
    extra_data: Vec<u8>
}

#[derive(Clone, Serialize, Deserialize)]
pub struct JsonCosmetic {
    id: u32,
    part_on: u32,
    extra_data: Vec<u8>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Robot {
    pub metadata: u64,
    pub bot_name: Vec<u8>,
    pub parts: Vec<Part>,
    pub cosmetics: Vec<Cosmetic>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Part {
    pub id: u32,
    pub pos_x: i8,
    pub pos_y: i8,
    pub pos_z: i8,
    pub rotation: u8,
    pub color_r: u8,
    pub color_g: u8,
    pub color_b: u8,
    pub alpha_channel: u8,
    pub extra_bytes: Vec<u8>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Cosmetic {
    pub id: u32,
    pub on_part: u32,
    pub extra_bytes: Vec<u8>
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct RobotMetadataInfo {
    pub name: String,
    pub cpu: i32,
    pub ranking: i32,
    pub mass: i32,
    pub cost: i32,
    pub primary_weapon: u32,
    pub secondary_weapon: u32
}

impl TryFrom<&[u8]> for Robot {
    type Error = std::io::Error;
    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        let mut blank = Robot::new();
        let mut file = Cursor::new(data);
        let mut buf4 = [0u8; 4];
        file.read_exact(&mut buf4)?;
        let magic = u32::from_be_bytes(buf4);
        if magic != ROBOT_MAGIC_NUMBER {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Magic number was invalid: {}", magic),
            ));
        }

        file.read_exact(&mut buf4)?;
        let version = u32::from_be_bytes(buf4);
        let res: Result<(), std::io::Error> = match version {
            1 => Robot::from_v1(&mut blank, &mut file),
            2 => Robot::from_v2(&mut blank, &mut file),
            3 => Robot::from_v3(&mut blank, &mut file),
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

impl From<JsonRobot> for Robot {
    fn from(file: JsonRobot) -> Self {
        let mut bot = Robot::new();
        bot.bot_name = file.name.into_bytes();
        bot.metadata = file.metadata;
        bot.parts = Vec::new();
        file.parts.into_iter().for_each( |part| {
            bot.parts.push(Part {
                id: part.id,
                pos_x: part.pos[0],
                pos_y: part.pos[1],
                pos_z: part.pos[2],
                rotation: part.rot,
                color_r: part.color[0],
                color_g: part.color[1],
                color_b: part.color[2],
                alpha_channel: part.alpha,
                extra_bytes: part.extra_data
            })
        });
        file.cosmetics.into_iter().for_each(|part| {
            bot.cosmetics.push(Cosmetic {
                id: part.id,
                on_part: part.part_on,
                extra_bytes: part.extra_data
            });
        });
        bot
    }
}

impl Robot {
    fn from_v1(inv: &mut Robot, file: &mut Cursor<&[u8]>) -> Result<(), std::io::Error> {
        let mut buf4 = [0u8; 4];
        let mut buf2 = [0u8; 2];
        let mut buf1 = [0u8; 1];
        file.seek(std::io::SeekFrom::Current(8))?; // skip metadata

        inv.metadata = 0;
        file.read_exact(&mut buf1)?;
        let name_size = u8::from_be_bytes(buf1);

        let mut bufname = vec!(0u8; name_size.into());
        file.read_exact(&mut bufname)?;
        inv.bot_name = bufname;

        file.read_exact(&mut buf4)?;
        let num_elems = u32::from_be_bytes(buf4);
        for _ in 0..num_elems {

            file.read_exact(&mut buf1)?;
            let pos_x = i8::from_be_bytes(buf1);
            file.read_exact(&mut buf1)?;
            let pos_y = i8::from_be_bytes(buf1);
            file.read_exact(&mut buf1)?;
            let pos_z = i8::from_be_bytes(buf1);
            file.read_exact(&mut buf1)?;
            let rotation = u8::from_be_bytes(buf1);
            file.read_exact(&mut buf1)?;
            let col_r = u8::from_be_bytes(buf1);
            file.read_exact(&mut buf1)?;
            let col_g = u8::from_be_bytes(buf1);
            file.read_exact(&mut buf1)?;
            let col_b = u8::from_be_bytes(buf1);
            file.read_exact(&mut buf1)?;
          
            file.read_exact(&mut buf2)?;
            let part_id = u16::from_be_bytes(buf2);
            inv.parts.push( Part {
                id: part_id.into(), pos_x: pos_x, pos_y: pos_y, pos_z: pos_z, rotation: rotation,
                color_r: col_r, color_g: col_g, color_b: col_b, alpha_channel: 0, extra_bytes: Vec::new()
            });
        }
        Ok(())
    }

    fn from_v2(inv: &mut Robot, file: &mut Cursor<&[u8]>) -> Result<(), std::io::Error> {
        let offset = file.position();
        let mut whole = Vec::new();
        file.read_to_end(&mut whole)?;
        let mut md5hash = Md5::new();
        md5hash.update(&whole[0..whole.len()-16]);
        let res = md5hash.finalize();
        if &res[..] != &whole[whole.len()-16..] {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Bot hash did not match"));
        }

        let mut buf8 = [0u8; 8];
        let mut buf4 = [0u8; 4];
        let mut buf1 = [0u8; 1];
        file.seek(std::io::SeekFrom::Start(offset))?;

        file.read_exact(&mut buf8)?;
        inv.metadata = u64::from_be_bytes(buf8);

        file.read_exact(&mut buf1)?;
        let name_size = u8::from_be_bytes(buf1);

        let mut bufname = vec!(0u8; name_size.into());
        file.read_exact(&mut bufname)?;
        inv.bot_name = bufname;


        file.read_exact(&mut buf4)?;
        let num_elems = u32::from_be_bytes(buf4);
        for i in 0..num_elems {
            file.read_exact(&mut buf4)?;
            let part_id = u32::from_be_bytes(buf4);
            file.read_exact(&mut buf1)?;
            let pos_x = i8::from_be_bytes(buf1);
            file.read_exact(&mut buf1)?;
            let pos_y = i8::from_be_bytes(buf1);
            file.read_exact(&mut buf1)?;
            let pos_z = i8::from_be_bytes(buf1);
            file.read_exact(&mut buf1)?;
            let rotation = u8::from_be_bytes(buf1);
            file.read_exact(&mut buf1)?;
            let col_r = u8::from_be_bytes(buf1);
            file.read_exact(&mut buf1)?;
            let col_g = u8::from_be_bytes(buf1);
            file.read_exact(&mut buf1)?;
            let col_b = u8::from_be_bytes(buf1);
            file.read_exact(&mut buf1)?;
            let alpha = u8::from_be_bytes(buf1);
            file.read_exact(&mut buf1)?;
            let extradata_size = u8::from_be_bytes(buf1);
            if extradata_size > MAX_EXTRADATA_SIZE {
                return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("Block {}: Extra data region can only be 64 bytes long (was {})", i, extradata_size)));
            }
            let mut bytes = vec!(0u8; extradata_size.into());
            file.read_exact(&mut bytes)?;
            inv.parts.push( Part {
                id: part_id, pos_x: pos_x, pos_y: pos_y, pos_z: pos_z, rotation: rotation,
                color_r: col_r, color_g: col_g, color_b: col_b, alpha_channel: alpha, extra_bytes: bytes
            });
        }
        Ok(())
    }

    fn from_v3(inv: &mut Robot, file: &mut Cursor<&[u8]>) -> Result<(), std::io::Error> {
        let mut buf4 = [0u8; 4];
        let mut buf1 = [0u8; 1];

        Robot::from_v2(inv, file)?;

        file.read_exact(&mut buf4)?;
        let num_elems = u32::from_be_bytes(buf4);
        for i in 0..num_elems {
            file.read_exact(&mut buf4)?;
            let cosm_id = u32::from_be_bytes(buf4);
            file.read_exact(&mut buf4)?;
            let on_id = u32::from_be_bytes(buf4);

            file.read_exact(&mut buf1)?;
            let extradata_size = u8::from_be_bytes(buf1);
            if extradata_size > MAX_EXTRADATA_SIZE {
                return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("Cosmetic {}: Extra data region can only be 64 bytes long (was {})", i, extradata_size)));
            }
            let mut bytes = vec!(0u8; extradata_size.into());
            file.read_exact(&mut bytes)?;
            inv.cosmetics.push( Cosmetic {
                id: cosm_id, on_part: on_id, extra_bytes: bytes
            });
        }
        Ok(())
    }

    pub fn new() -> Robot {
        Robot {
            metadata: 0u64,
            bot_name: "robot".to_owned().into_bytes(),
            parts: Vec::new(),
            cosmetics: Vec::new()
        }
    }

    pub fn compile(self: &Robot) -> Result<Vec<u8>, std::io::Error> {
        let mut file = Cursor::new(Vec::new());
        file.write_all(&u32::to_be_bytes(ROBOT_MAGIC_NUMBER))?; // "57A7F11E" STATFILE magic number
        file.write_all(&u32::to_be_bytes(2))?;// TODO fix CURRENT_VERSION))?;
        file.write_all(&u64::to_be_bytes(self.metadata))?;
        if self.bot_name.len() > u8::MAX.into() {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Bot name can only be 255 bytes long"))
        }
        file.write_all(&u8::to_be_bytes(self.bot_name.len() as u8))?;
        file.write_all(&self.bot_name)?;

        file.write_all(&u32::to_be_bytes(self.parts.len() as u32))?;

        for elem in self.parts.iter() {
            file.write_all(&u32::to_be_bytes(elem.id))?;
            file.write_all(&i8::to_be_bytes(elem.pos_x))?;
            file.write_all(&i8::to_be_bytes(elem.pos_y))?;
            file.write_all(&i8::to_be_bytes(elem.pos_z))?;
            file.write_all(&u8::to_be_bytes(elem.rotation))?;
            file.write_all(&u8::to_be_bytes(elem.color_r))?;
            file.write_all(&u8::to_be_bytes(elem.color_g))?;
            file.write_all(&u8::to_be_bytes(elem.color_b))?;
            file.write_all(&u8::to_be_bytes(elem.alpha_channel))?;
            if elem.extra_bytes.len() > MAX_EXTRADATA_SIZE.into() {
                return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Extra data region can only be 64 bytes long"))
            }
            file.write_all(&u8::to_be_bytes(elem.extra_bytes.len() as u8))?;
            file.write_all(&elem.extra_bytes)?;
        }
/*
        for elem in self.cosmetics.iter() {
            file.write_all(&u32::to_be_bytes(elem.id))?;
            file.write_all(&u32::to_be_bytes(elem.on_part))?;
            if elem.extra_bytes.len() > MAX_EXTRADATA_SIZE.into() {
                return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Extra data region can only be 64 bytes long"))
            }
            file.write_all(&u8::to_be_bytes(elem.extra_bytes.len() as u8))?;
            file.write_all(&elem.extra_bytes)?;
        }
*/
        let mut md5hash = Md5::new();
        let mut file_sans_hash = Vec::new();
        file.seek(std::io::SeekFrom::Start(8))?;
        file.read_to_end(&mut file_sans_hash)?;
        md5hash.update(file_sans_hash);
        let result = md5hash.finalize();
        file.write_all(&result)?; // should have read to end of file, so cursor is already at end
        Ok(file.into_inner())
    }
}
