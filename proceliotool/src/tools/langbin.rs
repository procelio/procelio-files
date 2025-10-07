use image::GenericImageView;

use itertools::Itertools;
use procelio_files::files::localization::localization::{self, TextElement};
use std::collections::HashSet;
use std::io::{Read, Write, BufRead};

pub struct LangBinTool {

}

impl super::ProcelioCLITool for LangBinTool {
    fn command(&self) -> &'static str {
        "lang"
    }

    fn usage(&self) {
        println!("path/to/language/folder [path/to/entries]");
        println!("    creates a new language (if given folder is empty), OR converts JSON/PNG image data to a compiled language file");
    }

    fn tool(&self, args: Vec<String>) {
        tool_impl(args)
    }
}

fn tool_impl(args: Vec<String>) {
    let mut args = args.into_iter();
    let arg = args.next().unwrap();
    let config = args.next().unwrap_or("entries.txt".to_owned());
    let source = std::path::Path::new(&arg);
    let mut cfgrawpath = std::path::Path::new(&config).to_owned();
    if !cfgrawpath.exists() {
        cfgrawpath = source.join("..").join(&config);
    }
    if !cfgrawpath.exists() {
        eprintln!("Config file does not exist {}", cfgrawpath.display());
        return;
    }
    let entries: std::collections::HashSet<String> = std::io::BufReader::new(std::fs::File::open(cfgrawpath).unwrap()).lines().map(|x|x.unwrap()).collect();

    if !source.exists() || !source.join("language.json").exists() || !source.join("image.png").exists() {
        println!("Language data does not exist; creating");
        let cdir = std::fs::create_dir_all(source);
        if cdir.is_err() {
            eprintln!("Path {} not valid for directory creation: {:#?}", source.to_string_lossy(), cdir.err().unwrap());
            return;
        }
        let ff1 = std::fs::File::create(source.join("language.json"));
        if ff1.is_err() {
            eprintln!("Unable to save langfile: {:#?}", ff1.err().unwrap());
            return;
        }

        let mut t = localization::Translation::new();
        for elem in entries {
            t.language_elements.push(TextElement::new(elem));
        }
        let serdestr = serde_json::to_string_pretty(&t).unwrap();
        let written = ff1.unwrap().write_all(serdestr.as_bytes());
        if let Err(e) = written {
            eprintln!("Unable to write langfile: {:#?}", e);
            return;
        }

        let imgsize = localization::lang_image_size();
        let mut img = image::ImageBuffer::new(imgsize.0.into(), imgsize.1.into());
        for (x, y, pixel) in img.enumerate_pixels_mut() {
            let r: f32 = x as f32 * 255.0 / (imgsize.0 as f32);
            let g: f32 = y as f32 * 255.0 / (imgsize.1 as f32);
            let b: f32 =(x + y) as f32 * 255.0 / ((imgsize.0 + imgsize.1) as f32);
            *pixel = image::Rgba([r as u8, g as u8, b as u8, 255_u8]);
        }
        let save = img.save(source.join("image.png"));
        if let Err(e) = save {
            eprintln!("Failed to create translation image: {:#?}", e);
            return;
        }
        return;
    }

    let mut imgbytes = Vec::new();

    let imgfile = std::fs::File::open(source.join("image.png"));

    let mut imgfile = match imgfile {
       Err(e) => { eprintln!("Failed to open image: {:#?}", e); return; },
       Ok(f) => f
    };
    if let Err(e) =  imgfile.read_to_end(&mut imgbytes) {
        eprintln!("Failed to read image: {:#?}", e); 
        return; 
    }
    let img = match image::load_from_memory_with_format(&imgbytes, image::ImageFormat::Png) {
        Err(e) => {eprintln!("Unable to parse image: {:#?}", e); return;},
        Ok(i) => i
    };

    if img.width() != localization::lang_image_size().0 as u32 || img.height() != localization::lang_image_size().1 as u32 {
        eprintln!("Image not the correct size");
        return;
    }

    let mut imgoutbytes = Vec::new();
    // Flip direction to get row-major
    for y in 0..localization::lang_image_size().1 {
        for x in 0..localization::lang_image_size().0 {
            let pix = img.get_pixel(x.into(), y.into());
            imgoutbytes.push(pix[0]);
            imgoutbytes.push(pix[1]);
            imgoutbytes.push(pix[2]);
            imgoutbytes.push(pix[3]);
        }
    }

    let json = std::fs::read_to_string(source.join("language.json"));
    let json = match json {
        Err(e) => { eprintln!("Failed to read language data: {:#?}", e); return; },
        Ok(o) => o
    };

    let mut translate: localization::Translation = match serde_json::from_str(&json) {
        Err(e) => { eprintln!("File was not valid JSON: {:#?}", e); return; },
        Ok(o) => o
    };

    let existing: HashSet<String> = translate.language_elements.iter().map(|x|x.name.to_owned()).collect();
    let elen = entries.len();
    if elen != existing.len() || elen != entries.union(&existing).count()
    {
        let ff1 = std::fs::File::create(source.join("language.json"));
        if ff1.is_err() {
            eprintln!("Unable to save langfile: {:#?}", ff1.err().unwrap());
            return;
        }

        for elem in &entries {
            if !existing.contains(elem) {
                translate.language_elements.push(TextElement::new(elem.to_owned()))
            }
        }

        translate.language_elements = translate.language_elements.into_iter()
            .filter(|x|entries.contains(&x.name))
            .sorted_by(|a,b| Ord::cmp(&a.name,&b.name))
            .collect();
        
        let serdestr = serde_json::to_string_pretty(&translate).unwrap();
        let written = ff1.unwrap().write_all(serdestr.as_bytes());
        if let Err(e) = written {
            eprintln!("Unable to write langfile: {:#?}", e);
            return;
        }
        println!("Updated langfile");
        return;
    }

    translate.language_image = imgoutbytes;

    let output: Vec<u8> = match translate.compile() {
        Err(e) => {eprintln!("Failed to compile file: {:#?}", e); return; },
        Ok(o) => o
    };

    let name = format!("{}.lang", translate.anglicized_name);

    if let Err(e) = std::fs::write(source.join(&name), &output) {
        eprintln!("Failed to save compiled language: {:#?}", e);
    }
}