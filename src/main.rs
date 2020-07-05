mod files;
use std::io::prelude::*;
use std::io::{Read, Write};
use std::convert::TryFrom;
fn usage() {
    println!("{} COMMAND ARGS", std::env::current_exe().unwrap_or(std::path::PathBuf::from("./program")).display());
    println!("Commands: ");
    println!("  statbin path/to/json [path/to/bin]: converts a json stat file to binary");
    println!("  dump path/to/file: prints out the contents of a binary file in readable form");
}

fn statbin_usage() {
    println!("statbin path/to/json.json [path/to/bin]");
    println!("  converts a json partstats file (at given path) to the binary representation");
    println!("  if no path/to/bin is supplied, defaults to 'path/to/json.stats'");
    println!("  either way, the resultant file is suitable for being served");
}

fn statbin(mut args: std::env::Args) {
    let arg = args.next().unwrap_or("--help".to_owned());
    if arg == "--help" || arg == "-h" {
        statbin_usage();
        return;
    }

    let source = std::path::Path::new(&arg);
    let dst = args.next();
    let destination = match dst {
        None => source.with_extension("stat"),
        Some(e) => std::path::PathBuf::from(e)
    };

    let file = std::fs::File::open(source);
    if let Err(e) = file {
        println!("Could not open {}: {}", source.display(), e);
        return;
    }
    let mut file = file.unwrap();
    let mut file_contents = std::vec::Vec::new();
    let nr = file.read_to_end(&mut file_contents);
    if let Err(e) = nr {
        println!("Could not read {}: {}", source.display(), e);
        return;
    }
    let statfile : files::statfile::statfile::JsonStatsFile = serde_json::from_slice(&file_contents).unwrap();
    let binarystatfile : files::statfile::statfile::StatsFile = statfile.into();
    let res = binarystatfile.compile();
    match res {
        Err(e) => { println!("Unable to compile statfile: {}", e)},
        Ok(data) => {
            let ff = std::fs::File::create(destination);
            if ff.is_err() {
                println!("Unable to save statfile: {}", ff.err().unwrap());
                return;
            }
            let written = ff.unwrap().write_all(&data);
            if let Err(e) = written {
                println!("Unable to save statfile: {}", e);
                return;
            }
         }
    }
    println!("Statfile successfully compiled");
}

fn dump_usage() {
    println!("dump path/to/file");
    println!("  reads the given binary file and tries to print a JSON-deserialized form of it");
}

fn dump(mut args: std::env::Args) {
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
        files::statfile::statfile::STATFILE_MAGIC_NUMBER => {
            br.seek(std::io::SeekFrom::Start(0)).unwrap();
            let mut buf = std::vec::Vec::new();
            br.read_to_end(&mut buf).unwrap();
            let sf = files::statfile::statfile::StatsFile::try_from(buf.as_slice());
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
        _ => { println!("Invalid filetype! Only supports [stats]"); return;}
    }
}

fn main() {
    let mut args = std::env::args();
    args.next(); // executable name, dispose
    let tool = args.next();
    if let None = tool {
        usage();
        return;
    }
    let tool = tool.unwrap();

    match tool.as_str() {
        "help" => {usage(); return;},
        "statbin" => statbin(args),
        "dump" => dump(args),
        _ => {usage(); return;}
    };


}