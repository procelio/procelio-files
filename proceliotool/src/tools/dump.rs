use procelio_files::files::*;
use std::io::prelude::*;
use std::io::Read;
use std::convert::TryFrom;

pub struct DumpTool {

}

impl super::ProcelioCLITool for DumpTool {
    fn command(&self) -> &'static str {
        "dump"
    }

    fn usage(&self) {
        println!("path/to/file");
        println!("    reads the given binary file and tries to print a JSON-deserialized form of it");
    }

    fn tool(&self, args: Vec<String>) {
        tool_impl(args)
    }
}

fn tool_impl(args: Vec<String>) {
    let file = &args[0];
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
                    match serde_json::to_string_pretty(&s) {
                        Err(e2) => {println!("Unable to serialize: {}", e2);},
                        Ok(s2) => {println!("{}", s2);}
                    }

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
                    match serde_json::to_string_pretty(&inventory::JsonInventory::from(&s)) {
                        Err(e2) => {println!("Unable to serialize: {}", e2);},
                        Ok(s2) => {println!("{}", s2);}
                    }

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
                    match serde_json::to_string_pretty(&robot::JsonRobot::from(s)) {
                        Err(e2) => {println!("Unable to serialize: {}", e2);},
                        Ok(s2) => {println!("{}", s2);}
                    }

                }
            }
        },
        localization::localization::LOCALIZATION_MAGIC_NUMBER => {
            br.seek(std::io::SeekFrom::Start(0)).unwrap();
            let mut buf = std::vec::Vec::new();
            br.read_to_end(&mut buf).unwrap();
            let sf = localization::localization::Translation::try_from(buf.as_slice());
            match sf {
                Err(e) => {println!("Unable to parse file: {}", e);},
                Ok(s) => {
                    match serde_json::to_string_pretty(&s) {
                        Err(e2) => {println!("Unable to serialize: {}", e2);},
                        Ok(s2) => {println!("{}", s2);}
                    }

                }
            }
        },
        _ => { println!("Invalid filetype! Only supports [stats, inventory, robot, translation]");}
    }
}