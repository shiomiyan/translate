use reqwest::header::AUTHORIZATION;
use reqwest::header::HeaderMap;
use reqwest::Client;
use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use std::env;

#[derive(Deserialize, Debug)]
struct Response {
    translations: Vec<Translation>,
}

#[derive(Deserialize, Debug)]
struct Translation {
    detected_source_language: String,
    text: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Parameter {
    text: String,
    target_lang: String,
}

async fn build<T>(url: String, parameter: Parameter) -> Result<T, StatusCode>
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
    let req = client.get(url).headers(headers).query(&parameter);
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

async fn translate<T>(parameter: Parameter) -> Result<T, StatusCode>
where
    T: DeserializeOwned,
{
    let url = String::from("https://api-free.deepl.com/v2/translate");
    build(url, parameter).await
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn translate_test() {
        let parameter = Parameter {
            text: "こんにちは、世界！".to_string(),
            target_lang: "EN".to_string(),
        };
        let resp: Response = translate(parameter).await.unwrap();
        dbg!(resp);
    }
}


