use procelio_files::files::tech::*;
use std::io::{Read, Write};

pub struct TechBinTool {

}

impl super::ProcelioCLITool for TechBinTool {
    fn command(&self) -> &'static str {
        "techbin"
    }

    fn usage(&self) {
        println!("path/to/json.json [path/to/bin]");
        println!("    converts a json tech file (at given path) to the binary representation");
        println!("    if no path/to/bin is supplied, defaults to 'path/to/json.tech'");
    }

    fn tool(&self, args: Vec<String>) {
        tool_impl(args)
    }
}

fn tool_impl(args: Vec<String>) {
    let arg = &args[0];
    let source = std::path::Path::new(&arg);
    let dst = args.get(1);
    let destination = match dst {
        None => source.with_extension("tech"),
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
    let json : TechTree = serde_json::from_slice(&file_contents).unwrap();
    let res = json.compile();
    match res {
        Err(e) => { println!("Unable to compile techfile: {}", e)},
        Ok(data) => {
            let ff = std::fs::File::create(destination);
            if ff.is_err() {
                println!("Unable to save techfile: {}", ff.err().unwrap());
                return;
            }
            let written = ff.unwrap().write_all(&data);
            if let Err(e) = written {
                println!("Unable to save techfile: {}", e);
                return;
            }
         }
    }
    println!("Techfile successfully compiled");
}