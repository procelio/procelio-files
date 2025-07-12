use procelio_files::files::stats::statfile;
use std::io::{Read, Write};

pub struct StatBinTool {

}

impl super::ProcelioCLITool for StatBinTool {
    fn command(&self) -> &'static str {
        "statbin"
    }

    fn usage(&self) {
        println!("path/to/json.json [path/to/bin]");
        println!("    converts a json partstats file (at given path) to the binary representation");
        println!("    if no path/to/bin is supplied, defaults to 'path/to/json.stats'");
        println!("    either way, the resultant file is suitable for being served");
    }

    fn tool(&self, args: Vec<String>) {
        tool_impl(args)
    }
}

fn tool_impl(args: Vec<String>) {
    let mut args = args.into_iter();
    let arg = args.next().unwrap();
    let source = std::path::Path::new(&arg);
    let dst = args.next();
    let destination = match dst {
        None => source.with_extension("stats"),
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
    let statfile : statfile::JsonStatsFile = serde_json::from_slice(&file_contents).unwrap();
    let binarystatfile : statfile::StatsFile = statfile.into();
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