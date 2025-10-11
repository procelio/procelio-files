use procelio_files::files::inventory::Inventory;
use procelio_files::files::inventory::JsonInventory;
use procelio_files::files::localization::localization::Translation;
use procelio_files::files::robot::JsonRobot;
use procelio_files::files::robot::Robot;
use procelio_files::files::stats::statfile;
use procelio_files::files::stats::statfile::StatsFile;
use procelio_files::files::tech::TechTree;
use procelio_files::files::*;
use serde::Serialize;
use std::fmt::Debug;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
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

fn dump<T: for<'a> TryFrom<&'a [u8]>, A: Serialize, N>(mut br: BufReader<File>, tf: N) where N: FnOnce(T) -> A{
    br.seek(std::io::SeekFrom::Start(0)).unwrap();
    let mut buf = std::vec::Vec::new();
    br.read_to_end(&mut buf).unwrap();
    let sf = T::try_from(buf.as_slice());
    match sf {
        Err(e) => {println!("Unable to parse file");},
        Ok(s) => {
            match serde_json::to_string_pretty(&tf(s)) {
                Err(e2) => {println!("Unable to serialize: {}", e2);},
                Ok(s2) => {println!("{}", s2);}
            }

        }
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
            dump(br, |x: StatsFile| x);
        },
        inventory::INVENTORY_MAGIC_NUMBER => {
            dump(br, |x: Inventory| JsonInventory::from(&x));
        },
        robot::ROBOT_MAGIC_NUMBER => {
            dump(br, |x: Robot| JsonRobot::from(x));
        },
        localization::localization::LOCALIZATION_MAGIC_NUMBER => {
            dump(br, |x: Translation| x);
        },
        tech::TECHTREE_MAGIC_NUMBER => {
            dump(br, |x: TechTree| x);
        },
        _ => { println!("Invalid filetype! Only supports [stats, inventory, robot, translation, tech]");}
    }
}