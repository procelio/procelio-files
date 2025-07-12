use proceliotool::tools::{self, ProcelioCLITool};

fn main() {
    let mut args = std::env::args();
    let executable = args.next().unwrap(); // executable name, dispose
    let tool = args.next().unwrap_or("--help".to_owned());

    if tool == "version" {
        println!("2025.7.12");
        return;
    }

    let tools: Vec<Box<dyn ProcelioCLITool>> = vec!(
        Box::new(tools::botbin::BotBinTool {}),
        Box::new(tools::invbin::InvBinTool {}),
        Box::new(tools::langbin::LangBinTool {}),
        Box::new(tools::statbin::StatBinTool {}),
        Box::new(tools::botmgmt::BotMgmtTool {}),
        Box::new(tools::diff::DiffTool {}),
        Box::new(tools::dump::DumpTool {}),
        Box::new(tools::zip::ZipTool {}),
        Box::new(tools::patch::PatchTool {}),
        Box::new(tools::reserialize::ReserializeTool {}),
        Box::new(tools::chatbroadcast::ChatBroadcastTool {})
    );

    let matched = tools.iter().find(|&x| x.as_ref().command() == tool);

    if let Some(x) = matched {
        if std::env::args().any(|x| x == "--help" || x == "-h") {
            print!("{} {} ", executable, tool);
            x.as_ref().usage();
        } else { 
            x.tool(args.collect());
        }
        return;
    }

    // Usage
    let printable = format!("{}", std::env::current_exe().unwrap_or(std::path::PathBuf::from("./proceliotool.exe")).display());
    println!("{} commands:", printable);
    for tool in tools {
        print!("  {} ", tool.command());
        tool.usage();
        println!();
    }
}