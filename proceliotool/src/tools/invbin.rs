use procelio_files::files::inventory::inventory::*;
use std::io::{Read, Write};

pub struct InvBinTool {

}

impl super::ProcelioCLITool for InvBinTool {
    fn command(&self) -> &'static str {
        "invbin"
    }

    fn usage(&self) {
        println!("path/to/json.json [path/to/bin]");
        println!("    converts a json inventory file (at given path) to the binary representation");
        println!("    if no path/to/bin is supplied, defaults to 'path/to/json.inventory'");
        println!("    either way, the resultant file is suitable for being loaded into a player account");
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
        None => source.with_extension("inventory"),
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
    let invfile : JsonInventory = serde_json::from_slice(&file_contents).unwrap();
    let binaryinvfile : Inventory = invfile.into();
    let res = binaryinvfile.compile();
    match res {
        Err(e) => { println!("Unable to compile inventoryfile: {}", e)},
        Ok(data) => {
            let ff = std::fs::File::create(destination);
            if ff.is_err() {
                println!("Unable to save inventoryfile: {}", ff.err().unwrap());
                return;
            }
            let written = ff.unwrap().write_all(&data);
            if let Err(e) = written {
                println!("Unable to save inventoryfile: {}", e);
                return;
            }
         }
    }
    println!("Inventoryfile successfully compiled");
}