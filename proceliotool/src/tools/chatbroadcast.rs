use serde::Deserialize;

pub struct ChatBroadcastTool {

}

impl super::ProcelioCLITool for ChatBroadcastTool {
    fn command(&self) -> &'static str {
        "broadcast"
    }

    fn usage(&self) {
        println!("MESSAGE ADMIN_TOKEN BROADCAST_TOKEN");
        println!("    sends a broadcast message to all chat home servers");
    }

    fn tool(&self, args: Vec<String>) {
        tool_impl(args)
    }
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

fn tool_impl(args: Vec<String>) {
    let message = &args[0];
    let admin_token = &args[1];
    let broadcast_token = &args[2];

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

        client.post(format!("https://{}:{}/admin/broadcast", conn.connection[0].replace("wss://", "").replace("ws://", ""), conn.port))
            .body(message.clone())
            .bearer_auth(&broadcast_token)
            .send().unwrap();
        println!("  successfully!");
    }
}