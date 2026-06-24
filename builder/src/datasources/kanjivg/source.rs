use std::io::{Cursor, Read, Seek};
use zip::ZipArchive;

const GITHUB_API_URL: &str = "https://api.github.com/repos/KanjiVG/kanjivg/releases/latest";

pub struct KanjiSvg {
    pub literal: String,
    pub svg: String,
}

pub struct KanjiVgSource;

impl KanjiVgSource {
    pub fn fetch_and_parse(&self) -> Vec<KanjiSvg> {
        println!("Fetching latest KanjiVG release info...");
        let release: serde_json::Value = reqwest::blocking::Client::new()
            .get(GITHUB_API_URL)
            .header("User-Agent", "shodoukan-db-builder")
            .send()
            .expect("failed to fetch KanjiVG release info")
            .json()
            .expect("failed to parse KanjiVG release JSON");

        let zip_url = release["assets"]
            .as_array()
            .expect("no assets in KanjiVG release")
            .iter()
            .find_map(|a| {
                let url = a["browser_download_url"].as_str()?;
                url.ends_with("-main.zip").then(|| url.to_string())
            })
            .expect("no -main.zip asset in KanjiVG release");

        println!("Downloading KanjiVG from {}...", zip_url);
        let bytes = reqwest::blocking::Client::new()
            .get(&zip_url)
            .header("User-Agent", "shodoukan-db-builder")
            .send()
            .expect("failed to download KanjiVG ZIP")
            .bytes()
            .expect("failed to read KanjiVG ZIP bytes");

        let results = Self::parse_zip(Cursor::new(bytes));
        println!("Parsed {} kanji SVGs", results.len());
        results
    }

    pub fn parse_zip<R: Read + Seek>(reader: R) -> Vec<KanjiSvg> {
        let mut archive = ZipArchive::new(reader).expect("failed to open KanjiVG ZIP");
        let mut results = Vec::new();

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).expect("failed to read ZIP entry");
            let name = file.name().to_string();

            let stem = match name.rfind('/') {
                Some(pos) => &name[pos + 1..],
                None => &name,
            };

            if !stem.ends_with(".svg") || stem.len() < 9 {
                continue;
            }

            let hex = &stem[..5];
            let codepoint = match u32::from_str_radix(hex, 16) {
                Ok(n) => n,
                Err(_) => continue,
            };
            let literal = match char::from_u32(codepoint) {
                Some(c) => c.to_string(),
                None => continue,
            };

            let mut svg = String::new();
            file.read_to_string(&mut svg).expect("failed to read SVG content");
            results.push(KanjiSvg { literal, svg });
        }

        results
    }
}
