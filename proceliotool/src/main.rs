use proceliotool::tools;

fn usage() {
    println!("{} COMMAND ARGS", std::env::current_exe().unwrap_or(std::path::PathBuf::from("./program")).display());
    println!("Commands: ");
    println!("  statbin path/to/json [path/to/bin]: converts a json stat file to binary");
    println!("  invbin path/to/json [path/to/bin]: converts a json inventory file to binary");
    println!("  botbin path/to/json [path/to/bin]: converts a json robot file to binary");
    println!("  lang path/to/folder: converts language data to binary file");
    println!("  dump path/to/file: prints out the contents of a binary file in readable form");
    println!("  reserialize path/to/file: update a file to the newest binary version of it");
    println!("  diff path/to/from path/to/to: Creates a patch between these two game builds");
    println!("  zip path/to/dir: Zip up a directory");
    println!("  botmgmt: launch bot manager")
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
        "statbin" => tools::statbin::tool(args),
        "invbin" => tools::invbin::tool(args),
        "botbin" => tools::botbin::tool(args),
        "dump" => tools::dump::tool(args),
        "lang" => tools::langbin::tool(args),
        "reserialize" => tools::reserialize::tool(args),
        "diff" => tools::diff::tool(args),
        "zip" => tools::zip::tool(args),
        "patch" => tools::patch::tool(args),
        "botmgmt" => tools::botmgmt::tool(args),
        _ => {usage(); return;}
    };


}