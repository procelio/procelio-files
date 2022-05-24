// Procelio Translation Tool
// Copyright Brennan Stein 2020
use std::vec::Vec;
use serde::{Serialize, Deserialize};
use std::default::Default;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::convert::TryFrom;

pub const LOCALIZATION_MAGIC_NUMBER: u32 = 0x10CA112E; // "LOCALIZE"
const CURRENT_VERSION: u32 = 2;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct TextColor {
    pub color: (u8, u8, u8)
}

// Used so serde doesn't serialize default text values (save vertical space)
fn is_default<T: PartialEq + Default>(elem: &T) -> bool {
    *elem == Default::default()
}

pub fn lang_image_bytes() -> usize {
    48 * 24 * 4
}

// the size of the langauge image (width, height). Magic numbers.
pub fn lang_image_size() -> (u16, u16) {
    (48, 24)
}


// All of the data for a single translated UI text element
#[derive(Clone, Serialize, Deserialize)]
pub struct TextElement {
    pub name: String,
    pub value: String,
    #[serde(default, skip_serializing_if = "is_default")]
    pub size: u16,
    #[serde(default, skip_serializing_if = "is_default")]
    pub bold: bool,
    #[serde(default, skip_serializing_if = "is_default")]
    pub italic: bool,
    #[serde(default, skip_serializing_if = "is_default")]
    pub underline: bool,
    #[serde(default, skip_serializing_if = "is_default")]
    pub strikethrough: bool,
    #[serde(default, skip_serializing_if = "is_default")]
    pub alignment: u8,
    #[serde(flatten, default, skip_serializing_if = "is_default")]
    pub color: Option<TextColor>
}

// The "full" data for a translation
#[derive(Clone, Serialize, Deserialize)]
pub struct Translation {
    pub anglicized_name: String,
    pub native_name: String,
    pub authors: String,
    pub version: u32,
    #[serde(skip)] 
    pub language_image: Vec<u8>, // RGBA in row-major order
    pub language_elements: Vec<TextElement>
}

impl TextElement {
    pub fn new(name: String) -> TextElement {
        TextElement {
            name: name,
            value: "".to_string(),
            size: 0,
            bold: false,
            italic: false,
            underline: false,
            strikethrough: false,
            alignment: 0,
            color: Default::default()
        }
    }
}



// Serialization functions for compiling a translation
impl Translation {
    pub fn new() -> Translation {
        Translation {
            anglicized_name: "Anglicized Name".to_owned(),
            native_name: "Local Name".to_owned(),
            authors: "Authors".to_owned(),
            version: CURRENT_VERSION,
            language_image: Vec::new(),
            language_elements: Vec::new()
        }
    }

    fn compile_elem(&self, file: &mut Cursor<Vec<u8>>, text: &TextElement) -> Result<(), std::io::Error> {
        let name = text.name.as_bytes();
        file.write_all(&u16::to_be_bytes(name.len() as u16))?;
        file.write_all(name)?;
        let value = text.value.as_bytes();
        file.write_all(&u16::to_be_bytes(value.len() as u16))?;
        file.write_all(value)?;

        file.write_all(&u16::to_be_bytes(text.size as u16))?;
        let mut modifications : u8 = 0;
        if text.bold {
            modifications |= 1;
        }
        if text.italic {
            modifications |= 2;
        }
        if text.underline {
            modifications |= 4;
        }
        if text.strikethrough {
            modifications |= 8;
        }
        file.write_all(&u8::to_be_bytes(modifications))?;
        file.write_all(&u8::to_be_bytes(text.alignment))?;
        if let Some(x) = &text.color {
            file.write_all(&u8::to_be_bytes(1))?;
            file.write_all(&u8::to_be_bytes(x.color.0))?;
            file.write_all(&u8::to_be_bytes(x.color.1))?;
            file.write_all(&u8::to_be_bytes(x.color.2))?;
        } else {
            file.write_all(&u8::to_be_bytes(0))?;
        }

        Ok(())
    }

    // Compile "this" down to a network-serializable form (see docs/localization.md for format)
    pub fn compile(self: &Translation) -> Result<Vec<u8>, std::io::Error> {
        let mut file = Cursor::new(Vec::new());
        file.write_all(&u32::to_be_bytes(LOCALIZATION_MAGIC_NUMBER))?;
        file.write_all(&u32::to_be_bytes(CURRENT_VERSION))?;
        let start_offset = file.position();
        file.seek(SeekFrom::Start(8 + start_offset))?; // two offsets
        println!("writing version at {}", file.position());
        file.write_all(&u32::to_be_bytes(self.version))?;
        let anam = self.anglicized_name.as_bytes();
        file.write_all(&u16::to_be_bytes(anam.len() as u16))?;
        file.write_all(anam)?;
        let nnam = self.native_name.as_bytes();
        file.write_all(&u16::to_be_bytes(nnam.len() as u16))?;
        file.write_all(nnam)?;
        let autt = self.authors.as_bytes();
        file.write_all(&u16::to_be_bytes(autt.len() as u16))?;
        file.write_all(autt)?;
     

        let pic_start = file.position();
        println!("Write pic at {} -- {}", pic_start, self.language_image.len());
        file.write_all(&self.language_image)?;
        let data_start = file.position();
        file.seek(SeekFrom::Start(start_offset))?;
        file.write_all(&u32::to_be_bytes(pic_start as u32))?;
        file.write_all(&u32::to_be_bytes(data_start as u32))?;
        file.seek(SeekFrom::Start(data_start))?;
          
        file.write_all(&u32::to_be_bytes(self.language_elements.len() as u32))?;
        println!("Writing elements at {:?}", file.position());
        for elem in &self.language_elements {
            self.compile_elem(&mut file, &elem)?;
        }

        file.seek(SeekFrom::Start(0))?;
        let mut out = Vec::new();
        file.read_to_end(&mut out)?;
        Ok(out)
    }

    fn from_v1(translate: &mut Translation, file: &mut Cursor<&[u8]>) -> Result<(), std::io::Error> {
        let mut buf4 = [0u8; 4];
        let mut buf2 = [0u8; 2];
        let mut buf1 = [0u8; 1];
        file.seek(std::io::SeekFrom::Current(8))?; // skip metadata
        
        println!("reading version at  {}", file.position());
        file.read_exact(&mut buf4)?;
        translate.version = u32::from_be_bytes(buf4);


        file.read_exact(&mut buf2)?;
        let name_size = u16::from_be_bytes(buf2);
        let mut bufname = vec!(0u8; name_size.into());
        file.read_exact(&mut bufname)?;
        translate.anglicized_name = String::from_utf8(bufname).unwrap();
        file.read_exact(&mut buf2)?;
        let name_size = u16::from_be_bytes(buf2);
        let mut bufname = vec!(0u8; name_size.into());
        file.read_exact(&mut bufname)?;
        translate.native_name = String::from_utf8(bufname).unwrap();
        file.read_exact(&mut buf2)?;
        let name_size = u16::from_be_bytes(buf2);
        let mut bufname = vec!(0u8; name_size.into());
        file.read_exact(&mut bufname)?;
        translate.authors = String::from_utf8(bufname).unwrap();

        println!("reading image at {} -- {}", file.position(), lang_image_bytes());
        let mut imgbuf = vec!(0u8; lang_image_bytes());
        file.read_exact(&mut imgbuf)?;
        translate.language_image = imgbuf;

        file.read_exact(&mut buf4)?;
        let n = u32::from_be_bytes(buf4);
        for _ in 0..n {
            file.read_exact(&mut buf2)?;
            let name_size = u16::from_be_bytes(buf2);
            let mut bufname = vec!(0u8; name_size.into());
            file.read_exact(&mut bufname)?;
            let name = String::from_utf8(bufname).unwrap();

            file.read_exact(&mut buf2)?;
            let name_size = u16::from_be_bytes(buf2);
            let mut bufname = vec!(0u8; name_size.into());
            file.read_exact(&mut bufname)?;
            let value = String::from_utf8(bufname).unwrap();

            file.read_exact(&mut buf2)?;
            let text_size = u16::from_be_bytes(buf2);

            file.read_exact(&mut buf1)?;
            let bold = (buf1[0] & 0x1) > 0;
            let italic = (buf1[0] & 0x2) > 0;
            let under = (buf1[0] & 0x4) > 0;
            let strike = (buf1[0] & 0x8) > 0;
            file.read_exact(&mut buf1)?;
            let algn = buf1[0];
            file.read_exact(&mut buf1)?;
            let r = buf1[0];
            file.read_exact(&mut buf1)?;
            let g = buf1[0];
            file.read_exact(&mut buf1)?;
            let b = buf1[0];

            translate.language_elements.push(TextElement {
                name: name,
                value: value,
                size: text_size,
                bold,
                italic,
                underline: under,
                strikethrough: strike,
                alignment: algn,
                color: Some(TextColor { color: (r, g, b) })
            });
        }
        Ok(())
    }

    fn from_v2(translate: &mut Translation, file: &mut Cursor<&[u8]>) -> Result<(), std::io::Error> {
        let mut buf4 = [0u8; 4];
        let mut buf2 = [0u8; 2];
        let mut buf1 = [0u8; 1];
        file.seek(std::io::SeekFrom::Current(8))?; // skip metadata
        
        println!("reading version at  {}", file.position());
        file.read_exact(&mut buf4)?;
        translate.version = u32::from_be_bytes(buf4);


        file.read_exact(&mut buf2)?;
        let name_size = u16::from_be_bytes(buf2);
        let mut bufname = vec!(0u8; name_size.into());
        file.read_exact(&mut bufname)?;
        translate.anglicized_name = String::from_utf8(bufname).unwrap();
        file.read_exact(&mut buf2)?;
        let name_size = u16::from_be_bytes(buf2);
        let mut bufname = vec!(0u8; name_size.into());
        file.read_exact(&mut bufname)?;
        translate.native_name = String::from_utf8(bufname).unwrap();
        file.read_exact(&mut buf2)?;
        let name_size = u16::from_be_bytes(buf2);
        let mut bufname = vec!(0u8; name_size.into());
        file.read_exact(&mut bufname)?;
        translate.authors = String::from_utf8(bufname).unwrap();

        println!("reading image at {} -- {}", file.position(), lang_image_bytes());
        let mut imgbuf = vec!(0u8; lang_image_bytes());
        file.read_exact(&mut imgbuf)?;
        translate.language_image = imgbuf;

        file.read_exact(&mut buf4)?;
        let n = u32::from_be_bytes(buf4);
        for _ in 0..n {
            file.read_exact(&mut buf2)?;
            let name_size = u16::from_be_bytes(buf2);
            let mut bufname = vec!(0u8; name_size.into());
            file.read_exact(&mut bufname)?;
            let name = String::from_utf8(bufname).unwrap();

            file.read_exact(&mut buf2)?;
            let name_size = u16::from_be_bytes(buf2);
            let mut bufname = vec!(0u8; name_size.into());
            file.read_exact(&mut bufname)?;
            let value = String::from_utf8(bufname).unwrap();

            file.read_exact(&mut buf2)?;
            let text_size = u16::from_be_bytes(buf2);

            file.read_exact(&mut buf1)?;
            let bold = (buf1[0] & 0x1) > 0;
            let italic = (buf1[0] & 0x2) > 0;
            let under = (buf1[0] & 0x4) > 0;
            let strike = (buf1[0] & 0x8) > 0;
            file.read_exact(&mut buf1)?;
            let algn = buf1[0];
            file.read_exact(&mut buf1)?;
            let color = if buf1[0] == 1 {
                file.read_exact(&mut buf1)?;
                let r = buf1[0];
                file.read_exact(&mut buf1)?;
                let g = buf1[0];
                file.read_exact(&mut buf1)?;
                let b = buf1[0];
                Some(TextColor { color: (r, g, b) })
            } else {
                None
            };


            translate.language_elements.push(TextElement {
                name: name,
                value: value,
                size: text_size,
                bold,
                italic,
                underline: under,
                strikethrough: strike,
                alignment: algn,
                color: color
            });
        }
        Ok(())
    }
}


impl TryFrom<&[u8]> for Translation {
    type Error = std::io::Error;
    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        let mut buf4 = [0u8; 4];
        let mut buf2 = [0u8; 2];
        let mut buf1 = [0u8; 1];
        let mut blank = Translation::new();
        let mut file = Cursor::new(data);
        file.read_exact(&mut buf4)?;
        let magic = u32::from_be_bytes(buf4);
        if magic != LOCALIZATION_MAGIC_NUMBER {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Magic number was invalid: {}", magic),
            ));
        }
        file.read_exact(&mut buf4)?;
        let version = u32::from_be_bytes(buf4);
        let res: Result<(), std::io::Error> = match version {
            1 => Translation::from_v1(&mut blank, &mut file),
            2 => Translation::from_v2(&mut blank, &mut file),
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