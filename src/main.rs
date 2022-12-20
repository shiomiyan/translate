use chrono::{DateTime, Utc};
use dotenvy::dotenv;
use reqwest::{
    header::{HeaderMap, AUTHORIZATION},
    Result,
};
use serde::{Deserialize, Serialize};
use std::{env, fs};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().expect(".env file not found.");
    let deepl_api_key = env::var("DEEPL_API_KEY").expect("DEEPL_API_KEY not found in your ENV.");

    // read csv glossary list
    let csv = fs::read_to_string("dict/glossary.csv").expect("Glossary.csv not found in ./dict");

    let args: Vec<String> = env::args().collect();
    let text_to_translate = args[1].as_str();

    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        format!("DeepL-Auth-Key {}", deepl_api_key).parse().unwrap(),
    );

    let client = reqwest::Client::new();

    let create_glossary = client
        .post("https://api.deepl.com/v2/glossaries")
        .headers(headers.clone())
        .form(&[
            ("name", "Tmp"),
            ("source_lang", "JA"),
            ("target_lang", "EN"),
            ("entries_format", "csv"),
            ("entries", csv.as_str()),
        ])
        .send()
        .await?
        .json::<CreateGlossary>()
        .await?;

    println!("Glossaries entry count: {:#?}", &create_glossary.entry_count);

    let glossary_id = create_glossary.glossary_id;

    let translate = client
        .post("https://api.deepl.com/v2/translate")
        .headers(headers.clone())
        .form(&[
            ("text", text_to_translate),
            ("source_lang", "JA"),
            ("target_lang", "EN"),
            ("glossary_id", &glossary_id),
        ])
        .send()
        .await?
        .json::<Translations>()
        .await?;

    println!("{:#?}", &translate);

    // 単語リストは既存のものを編集してくれないので、毎回後処理で消す
    client
        .delete(format!(
            "https://api.deepl.com/v2/glossaries/{}",
            &glossary_id
        ))
        .headers(headers)
        .send()
        .await?;

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateGlossary {
    glossary_id: String,
    ready: bool,
    name: String,
    source_lang: String,
    target_lang: String,
    creation_time: DateTime<Utc>,
    entry_count: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct Translations {
    translations: Vec<Translation>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Translation {
    text: String,
}
