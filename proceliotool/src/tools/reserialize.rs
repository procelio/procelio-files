use procelio_files::files::*;
use std::io::prelude::*;
use std::io::Read;
use std::convert::TryFrom;

fn dump_usage() {
    println!("reserialize path/to/file");
    println!("  reads the given file and rewrites it to be an up-to-date binary form");
}

pub fn tool(mut args: std::env::Args) {
    let file = args.next().unwrap_or("--help".to_owned());
    if file == "--help" || file == "-h" {
        dump_usage();
        return;
    }
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
        inventory::inventory::INVENTORY_MAGIC_NUMBER => {
            br.seek(std::io::SeekFrom::Start(0)).unwrap();
            let mut buf = std::vec::Vec::new();
            br.read_to_end(&mut buf).unwrap();
            let sf = inventory::inventory::Inventory::try_from(buf.as_slice());
            match sf {
                Err(e) => {println!("Unable to parse file: {}", e);},
                Ok(s) => {
                    std::fs::write(path, s.compile().unwrap()).unwrap();
                }
            }
        },
        robot::robot::ROBOT_MAGIC_NUMBER => {
            br.seek(std::io::SeekFrom::Start(0)).unwrap();
            let mut buf = std::vec::Vec::new();
            br.read_to_end(&mut buf).unwrap();
            let sf = robot::robot::Robot::try_from(buf.as_slice());
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