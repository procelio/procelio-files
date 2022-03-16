use crate::files::robot::robot;
use std::{io::{Read, Write, BufRead}, os::windows::prelude::MetadataExt};
use reqwest::StatusCode;
use serde::{Serialize, Deserialize};
use crate::files::robot::robot::Robot;
use std::convert::{From, TryFrom};
#[derive(Clone,Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: i64,
    pub username: String,
    pub xp: i64,
    pub currency: i64,
    pub premium_currency: i64,
    pub num_garages: i32,
    pub premium_expiration_timestamp: i64, // UTC MILLIS; NO LEAP SECONDS
    pub chat_tag: String
}

fn toutf8(data: &[u8]) -> String {
    std::str::from_utf8(data).unwrap_or("???").to_owned()
}

fn display(count: &str, bot: &Robot) -> String {
    let s1 = format!("{}: ", count);
    let name = std::str::from_utf8(&bot.bot_name).unwrap_or("???").chars().collect::<Vec<char>>();
    let name: String = if name.len() > 30 {
        format!("{}...", name.iter().take(27).collect::<String>())
    } else {
        name.iter().collect::<String>()
    };
    let ct = format!("({}) ", bot.parts.len());
    format!("{}{}{}{}", s1, name, std::iter::repeat(' ').take(41 - s1.len() - name.len() - ct.len()).collect::<String>(), ct)
}

pub fn tool(mut args: std::env::Args) {
    let mut userID: i32 = 1;
    let mut read_token: String = "".to_owned();
    let mut write_token: String = "".to_owned();
    let mut autobuy: bool = false;
    if std::env::consts::OS == "windows" {
        let mut path = dirs::home_dir().unwrap();
        path.push("AppData");
        path.push("LocalLow");
        path.push("Procul Games");
        path.push("Procelio");
        path.push("login.session");
        if let Ok(f) = std::fs::File::open(&path) {
            let lines = std::io::BufReader::new(f).lines().map(|x|x.unwrap()).collect::<Vec<String>>();
            userID = lines[1].parse().unwrap();
            read_token = lines[3].clone();
            write_token = lines[4].clone();
            println!("Procelio session file {:?} loaded", path);
        } else {
            println!("Procelio session file {:?} not found", path);
        }
    }
    while let Some(arg) = args.next() {
        if arg == "--help" {
            println!("--user [userID]");
            println!("--read [readToken]");
            println!("--write [writeToken]");
            println!("(if on windows and have played the game, pulled from login.session automatically)");
            println!("--autobuy: turn on autobuy");
            return;
        }
        if arg == "--user" {
            userID = args.next().unwrap().parse().unwrap();
        } else if arg == "--read" {
            read_token = args.next().unwrap();
        } else if arg == "--write" {
            write_token = args.next().unwrap();
        } else if arg == "--autobuy" {
            autobuy = true;
        }
    }

    let client = reqwest::blocking::Client::new();

    let user_data: UserResponse = client.get(format!("https://accounts.procelio.com:6676/users/{}", userID))
        .header("Authorization", format!("Bearer {}", read_token))
        .send().unwrap().json().unwrap();
    let mut server_bots: Vec<Robot> = Vec::new();
    let mut local_bots: Vec<Robot> = Vec::new();

    for i in 0..user_data.num_garages {
        let byts = client.get(format!("https://accounts.procelio.com:6676/users/{}/robots/{}", userID, i))
        .header("Authorization", format!("Bearer {}", read_token))
        .send().unwrap().bytes().unwrap();
        server_bots.push(Robot::try_from(&byts[..]).unwrap());
    }

    let mut ok = false;
    loop {
        clearscreen::clear().expect("failed to clear screen");
        if !ok {
            println!("Bad command!");
        }
        ok = false;
        local_bots.clear();
        
        for path in std::fs::read_dir(".").unwrap() {
            let mut file = std::fs::File::open(&path.unwrap().path()).unwrap();
            if file.metadata().unwrap().file_size() > 64000000 {
                continue;
            }
            let mut buf = Vec::new();
            file.read_to_end(&mut buf).unwrap();
            let bot = Robot::try_from(&buf[..]);
            if let Ok(b) = bot {
                local_bots.push(b);
            }
        }
        local_bots.sort_by(|x, y| x.bot_name.cmp(&y.bot_name));
        println!("Bot management | {} :: #{}", &user_data.username, &user_data.id);
        // 40 per
        println!("Serverside Bots                          | Local Bots                              ");
        println!("-----------------------------------------+-----------------------------------------");
        for i in 0..std::cmp::max(server_bots.len(), local_bots.len()) {
            if let Some(bot) = server_bots.get(i) {
                print!("{}|", display(&format!("{}", i), &bot));
            } else {
                print!("                                         |");
            }
           
            if let Some(bot) = local_bots.get(i) {
                let mut lbl = String::new();
                let mut j = i;
                if j == 0 {
                    lbl = "A".to_owned();
                }
                while j != 0 {
                    lbl = format!("{}{}", ('A' as u8 + (j % 26) as u8) as char, lbl);
                    j /= 26;
                }
                print!("{}", display(&lbl, &bot));
            }
            println!();
        }
        println!("-----------------------------------------------------------------------------------");
        println!("   | clear [#]");
        println!("   | download [#]");
        println!("   | upload [A] [#]");
        println!("   | quit");
        print!("> "); std::io::stdout().flush().unwrap();
        let mut buf = String::new();
        if let Err(_) = std::io::stdin().read_line(&mut buf) {
            break;
        }
        buf = buf.trim().to_string();
        let data: Vec<&str> = buf.split(' ').filter(|x| *x != "").collect();

        if buf == "quit" || buf == "exit" {
            break;
        }
        if buf.starts_with("clear") {
            let num = if let Some(n) = data.get(1).and_then(|x|x.parse::<u32>().ok()) {
                if n as usize > server_bots.len() {
                    println!("Bad command (A)!");
                    continue;
                }
                n
            } else {
                println!("Bad command (B)!");
                continue;
            };
            let bot = Robot::new();
            let data = client.patch(format!("https://accounts.procelio.com:6676/users/{}/robots/{}?autobuy={}", userID, num, autobuy))
                .header("Authorization", format!("Bearer {}", write_token))
                .body(bot.compile().unwrap())
                .send().unwrap();
            println!("{}", data.status());

            if data.status() == reqwest::StatusCode::OK {
                server_bots[num as usize] = bot;
            }
        }
        if buf.starts_with("download") {
            let serv = if let Some(b) = data.get(1).and_then(|x|x.parse::<u32>().ok()).and_then(|x|server_bots.get(x as usize)) {
                b
            } else {
                println!("Bad command!");
                continue;
            };
            println!("Saved to {}", format!("{}.bot",toutf8(&serv.bot_name)));
            std::fs::write(format!("{}.bot",toutf8(&serv.bot_name)), serv.compile().unwrap()).unwrap();
        }
        if buf.starts_with("upload") {
            let num = if let Some(n) = data.get(2).and_then(|x|x.parse::<u32>().ok()) {
                if (n < 0 || n as usize > server_bots.len()) {
                    println!("Bad command (A)!");
                    continue;
                }
                n
            } else {
                println!("Bad command (B)!");
                continue;
            };
            let slot = if let Some(g) = data.get(1) {
                let st = g.to_uppercase();
                let mut i: i32 = 0;
                for c in st.chars().rev() {
                    i *= 26;
                    i += (c as u8 - 'A' as u8) as i32;
                }
                if i < 0 || i as usize > local_bots.len() {
                    println!("Bad count {}", i);
                    continue;
                }
                i
            } else {
                println!("Bad command (C)!");
                continue;
            };
            
            let data = client.patch(format!("https://accounts.procelio.com:6676/users/{}/robots/{}?autobuy={}", userID, num, autobuy))
                .header("Authorization", format!("Bearer {}", write_token))
                .body(local_bots.get(slot as usize).unwrap().compile().unwrap())
                .send().unwrap();
            if data.status() == reqwest::StatusCode::OK {
                server_bots[num as usize] = local_bots.get(slot as usize).unwrap().clone();
            }
            println!("{}", data.status());
        }
        ok = true;
    }
}
