use serde::Deserialize;

fn chatbroadcast_usage() {
    println!("broadcast MESSAGE ADMIN_TOKEN BROADCAST_TOKEN");
    println!("  sends a broadcast message to all chat home servers");
}

#[derive(Deserialize)]
struct AdminResponse {
    pub id: String
}

#[derive(Deserialize)]
struct LookupResponse {
    pub connection: Vec<String>,
    pub port: u32
}

pub fn tool(mut args: std::env::Args) {
    let arg = args.next().unwrap_or("--help".to_owned());
    if arg == "--help" || arg == "-h" || args.len() != 2 {
        chatbroadcast_usage();
        return;
    }

    let message = arg;
    let admin_token = args.next().unwrap();
    let broadcast_token = args.next().unwrap();

    let client = reqwest::blocking::Client::new();

    let servers = client.get("https://chat-distrib.procelio.com:9678/admin/status")
        .bearer_auth(admin_token)
        .send().unwrap()
        .json::<Vec<AdminResponse>>().unwrap();

    for server in servers {
        println!("Processing server {}", server.id);

        let body = format!("\"{}\"", server.id);

        let conn = client.post("https://chat-distrib.procelio.com:9678/public/lookup").body(body)
            .send().unwrap()
            .json::<LookupResponse>().unwrap();

        println!("  with connection {}:{}", conn.connection[0], conn.port);

        client.post(format!("https://{}:{}/admin/broadcast", conn.connection[0].replace("ws://", ""), conn.port))
            .body(message.clone())
            .bearer_auth(&broadcast_token)
            .send().unwrap();
        println!("  successfully!");
    }
}