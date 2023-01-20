#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use chrono::{DateTime, Utc};
use eframe::egui::{self, CollapsingHeader};

use reqwest::{
    header::{HeaderMap, AUTHORIZATION},
    Result,
};
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use std::{fs, process::Command};

fn main() {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(390.0, 500.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Translate with DeepL",
        options,
        Box::new(|cc| Box::new(MyApp::new(cc))),
    )
}

#[derive(PartialEq)]
enum DeeplApiPlan {
    Free,
    Pro,
}

struct AppConfig {
    deepl_api_key: String,
    deepl_api_plan: DeeplApiPlan,
}

struct MyApp {
    app_config: AppConfig,
    input: String,
    translated: String,
    back_translated: String,
}

fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "my_font".to_owned(),
        egui::FontData::from_static(include_bytes!("../assets/NotoSansJP-Regular.otf")),
    );
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "my_font".to_owned());

    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .insert(0, "my_font".to_owned());

    ctx.set_fonts(fonts);
}

impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        setup_custom_fonts(&cc.egui_ctx);

        Self {
            app_config: AppConfig {
                deepl_api_key: "<API_KEY>:fx".to_owned(),
                deepl_api_plan: DeeplApiPlan::Free,
            },
            input: "".to_owned(),
            translated: "".to_owned(),
            back_translated: "".to_owned(),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Translator");
            ui.vertical(|ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut self.input)
                        .hint_text("input text to translate")
                        .desired_width(f32::INFINITY)
                        .font(egui::TextStyle::Monospace),
                );
                if ui.button("translate now").clicked() {
                    ui.spinner();
                    let csv = fs::read_to_string("./glossary.csv").unwrap();
                    deepl(self, &csv).unwrap();
                };
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut self.translated)
                        .hint_text("translation result")
                        .desired_width(f32::INFINITY)
                        .font(egui::TextStyle::Monospace),
                );
                ui.add(
                    egui::TextEdit::multiline(&mut self.back_translated)
                        .hint_text("re-translation from result")
                        .desired_width(f32::INFINITY)
                        .font(egui::TextStyle::Monospace),
                );
                if ui.button("copy translation result to clipboard").clicked() {
                    ui.output().copied_text = String::from(&self.translated);
                };
            });

            ui.separator();

            CollapsingHeader::new("Options")
                .default_open(false)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("DeepL API Plan:");
                        ui.selectable_value(
                            &mut self.app_config.deepl_api_plan,
                            DeeplApiPlan::Free,
                            "Free",
                        );
                        ui.selectable_value(
                            &mut self.app_config.deepl_api_plan,
                            DeeplApiPlan::Pro,
                            "Pro",
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label("Deepl API Key:");
                        ui.text_edit_singleline(&mut self.app_config.deepl_api_key);
                    });
                    ui.horizontal(|ui| {
                        let pwd = std::env::current_dir()
                            .unwrap()
                            .join("glossary.csv")
                            .display()
                            .to_string();
                        if ui.button("🖊 edit glossary.csv").clicked() {
                            let out = Command::new("notepad")
                                .arg(pwd)
                                .output()
                                .expect("Couldn't open glossary.csv");
                        }
                    })
                })
        });
    }
}

fn deepl(app: &mut MyApp, csv: &str) -> Result<()> {
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        format!("DeepL-Auth-Key {}", app.app_config.deepl_api_key)
            .parse()
            .unwrap(),
    );

    let client = reqwest::blocking::Client::new();

    let api_domain = match app.app_config.deepl_api_plan {
        DeeplApiPlan::Free => "api-free.deepl.com",
        DeeplApiPlan::Pro => "api.deepl.com",
    };

    let create_glossary = client
        .post(format!("https://{api_domain}/v2/glossaries"))
        .headers(headers.clone())
        .form(&[
            ("name", "Tmp"),
            ("source_lang", "JA"),
            ("target_lang", "EN"),
            ("entries_format", "csv"),
            ("entries", csv),
        ])
        .send()?
        .json::<CreateGlossary>()?;

    let glossary_id = create_glossary.glossary_id;

    let translate = client
        .post(format!("https://{api_domain}/v2/translate"))
        .headers(headers.clone())
        .form(&[
            ("text", app.input.as_str()),
            ("source_lang", "JA"),
            ("target_lang", "EN"),
            ("glossary_id", &glossary_id),
        ])
        .send()?
        .json::<Translations>()?;

    app.translated = translate.translations[0].text.to_string();

    // 単語リストは既存のものを編集してくれないので、毎回後処理で消す
    client
        .delete(format!("https://{api_domain}/v2/glossaries/{glossary_id}"))
        .headers(headers.clone())
        .send()?;

    // 英→日の逆翻訳をする
    let back_translate = client
        .post(format!("https://{api_domain}/v2/translate"))
        .headers(headers.clone())
        .form(&[
            ("text", translate.translations[0].text.as_str()),
            ("source_lang", "EN"),
            ("target_lang", "JA"),
        ])
        .send()?
        .json::<Translations>()?;

    app.back_translated = back_translate.translations[0].text.to_string();

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
