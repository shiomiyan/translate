use reqwest::header::AUTHORIZATION;
use reqwest::header::HeaderMap;
use reqwest::Client;
use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use serde::Deserialize;

use std::env;

mod translate_text;

#[derive(Deserialize, Debug)]
struct Usage {
    character_count: i32,
    character_limit: i32,
}

async fn build<T>(url: String) -> Result<T, StatusCode>
where
    T: DeserializeOwned,
{
    let auth_key = env::var("DEEPL_AUTH_KEY").expect("DEEPL_AUTH_KEY is not set");

    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        format!("DeepL-Auth-Key {}", &auth_key).parse().unwrap(),
    );

    let client = Client::new();
    let req = client.get(url).headers(headers);
    let response = req.send().await;

    match &response {
        Ok(r) => {
            if r.status() != StatusCode::OK {
                return Err(r.status());
            }
        }
        Err(e) => {
            if e.is_status() {
                return Err(e.status().unwrap());
            } else {
                return Err(StatusCode::BAD_REQUEST);
            }
        }
    }

    let content = response.unwrap().json::<T>().await;

    match content {
        Ok(s) => Ok(s),
        Err(e) => {
            println!("{:?}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

async fn usage<T>() -> Result<T, StatusCode>
where
    T: DeserializeOwned,
{
    build("https://api-free.deepl.com/v2/usage".to_string()).await
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn usage() {
        let resp: Usage = build("https://api-free.deepl.com/v2/usage".to_string()).await.unwrap();
        dbg!(resp);
    }
}

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let auth_key = env::var("DEEPL_AUTH_KEY").expect("DEEPL_AUTH_KEY is not set");
//     let client = Client::new();
//     let resp = client
//         .get("https://api-free.deepl.com/v2/usage")
//         .header(AUTHORIZATION, format!("DeepL-Auth-Key {}", &auth_key))
//         .send()
//         .await?
//         .text()
//         .await?;
//     let json: Usage = serde_json::from_str(&resp)?;
//     Ok(())
// }
