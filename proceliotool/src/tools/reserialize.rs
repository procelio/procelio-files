use procelio_files::files::*;
use std::io::prelude::*;
use std::io::Read;
use std::convert::TryFrom;

pub struct ReserializeTool {

}

impl super::ProcelioCLITool for ReserializeTool {
    fn command(&self) -> &'static str {
        "reserialize"
    }

    fn usage(&self) {
        println!("path/to/file");
        println!("    reads the given file and rewrites it to be an up-to-date binary form");
    }

    fn tool(&self, args: Vec<String>) {
        tool_impl(args)
    }
}

fn tool_impl(args: Vec<String>) {
    let mut args = args.into_iter();
    let file = args.next().unwrap();
    let path = std::path::Path::new(&file);
    let file = std::fs::File::open(path);
    if let Err(e) = file {
        println!("Unable to open {}: {}", path.display(), e);
        return;
    }
    let file = file.unwrap();
    let mut br = std::io::BufReader::new(file);
    let mut magicnum = [0u8; 4];
    let res = br.read_exact(&mut magicnum);
    if let Err(e) = res {
        println!("Unable to read file: {}", e);
        return;
    }

    let magicnum = u32::from_be_bytes(magicnum);
    match magicnum {
            stats::statfile::STATFILE_MAGIC_NUMBER => {
            br.seek(std::io::SeekFrom::Start(0)).unwrap();
            let mut buf = std::vec::Vec::new();
            br.read_to_end(&mut buf).unwrap();
            let sf = stats::statfile::StatsFile::try_from(buf.as_slice());
            match sf {
                Err(e) => {println!("Unable to parse file: {}", e);},
                Ok(s) => {
                    std::fs::write(path, s.compile().unwrap()).unwrap();
                }
            }
        },
        inventory::INVENTORY_MAGIC_NUMBER => {
            br.seek(std::io::SeekFrom::Start(0)).unwrap();
            let mut buf = std::vec::Vec::new();
            br.read_to_end(&mut buf).unwrap();
            let sf = inventory::Inventory::try_from(buf.as_slice());
            match sf {
                Err(e) => {println!("Unable to parse file: {}", e);},
                Ok(s) => {
                    std::fs::write(path, s.compile().unwrap()).unwrap();
                }
            }
        },
        robot::ROBOT_MAGIC_NUMBER => {
            br.seek(std::io::SeekFrom::Start(0)).unwrap();
            let mut buf = std::vec::Vec::new();
            br.read_to_end(&mut buf).unwrap();
            let sf = robot::Robot::try_from(buf.as_slice());
            match sf {
                Err(e) => {println!("Unable to parse file: {}", e);},
                Ok(s) => {
                    std::fs::write(path, s.compile().unwrap()).unwrap();
                }
            }
        },
        _ => { println!("Invalid filetype! Only supports [stats, inventory, robot]"); return;}
    }
}