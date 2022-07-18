use reqwest::header::AUTHORIZATION;
use reqwest::Client;
use serde::Deserialize;

const DEEPL_AUTH_KEY: &str = "AUTH_KEY_HERE";

#[derive(Deserialize, Debug)]
struct Usage {
    character_count: i32,
    character_limit: i32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let resp = client
        .get("https://api-free.deepl.com/v2/usage")
        .header(AUTHORIZATION, DEEPL_AUTH_KEY)
        .send()
        .await?
        .text()
        .await?;
    dbg!(&resp);
    let json: Usage = serde_json::from_str(&resp)?;
    println!("{:?}", &json);
    Ok(())
}
