use std::io::{Cursor, Read};
use encoding_rs::EUC_JP;
use zip::ZipArchive;

const KRADZIP_URL: &str = "http://ftp.edrdg.org/pub/Nihongo/kradzip.zip";

pub struct Radical {
    pub literal: String,
    pub strokes: u32,
}

pub struct KanjiRadical {
    pub kanji_literal: String,
    pub radical_literal: String,
}

pub struct RadkfileSource;

impl RadkfileSource {
    pub fn fetch_and_parse(&self) -> (Vec<Radical>, Vec<KanjiRadical>) {
        println!("Downloading kradzip.zip...");
        let bytes = reqwest::blocking::get(KRADZIP_URL)
            .expect("failed to fetch kradzip.zip")
            .bytes()
            .expect("failed to read kradzip.zip bytes");

        let mut archive = ZipArchive::new(Cursor::new(bytes))
            .expect("failed to open kradzip.zip");

        let mut all_radicals: Vec<Radical> = Vec::new();
        let mut all_kanji_radicals: Vec<KanjiRadical> = Vec::new();

        for i in 0..archive.len() {
            let mut entry = archive.by_index(i).expect("failed to read zip entry");
            let name = entry.name().to_lowercase();
            let basename = name.rsplit('/').next().unwrap_or(&name);

            if !basename.starts_with("radkfile") {
                continue;
            }

            let mut raw: Vec<u8> = Vec::new();
            entry.read_to_end(&mut raw).expect("failed to read zip entry bytes");

            let (decoded, _, _) = EUC_JP.decode(&raw);
            let (radicals, kanji_radicals) = Self::parse(&decoded);
            all_radicals.extend(radicals);
            all_kanji_radicals.extend(kanji_radicals);
        }

        println!(
            "Parsed {} radicals, {} kanji-radical pairs",
            all_radicals.len(),
            all_kanji_radicals.len()
        );
        (all_radicals, all_kanji_radicals)
    }

    pub fn parse(text: &str) -> (Vec<Radical>, Vec<KanjiRadical>) {
        let mut radicals: Vec<Radical> = Vec::new();
        let mut kanji_radicals: Vec<KanjiRadical> = Vec::new();
        let mut current_radical: Option<String> = None;

        for line in text.lines() {
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some(rest) = trimmed.strip_prefix('$') {
                let mut parts = rest.split_whitespace();
                let literal = match parts.next() {
                    Some(l) => l.to_string(),
                    None => continue,
                };
                let strokes: u32 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
                current_radical = Some(literal.clone());
                radicals.push(Radical { literal, strokes });
            } else if let Some(radical_literal) = &current_radical {
                for kanji in trimmed.split_whitespace() {
                    kanji_radicals.push(KanjiRadical {
                        kanji_literal: kanji.to_string(),
                        radical_literal: radical_literal.clone(),
                    });
                }
            }
        }

        (radicals, kanji_radicals)
    }
}
