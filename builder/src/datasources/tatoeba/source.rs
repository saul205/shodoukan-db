use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader, Cursor};
use bzip2::read::BzDecoder;
use core::infrastructure::sqlite::repository::TatoebaTranslation;
use tar::Archive;

const LINKS_URL: &str = "https://downloads.tatoeba.org/exports/links.tar.bz2";
const SENTENCES_URL: &str =
    "https://downloads.tatoeba.org/exports/per_language/{lang}/{lang}_sentences.tsv.bz2";

pub struct TatoebaSource;

impl TatoebaSource {
    pub fn fetch_translations(
        &self,
        known: &HashMap<String, i64>,
        langs: &[String],
    ) -> Vec<TatoebaTranslation> {
        if known.is_empty() || langs.is_empty() {
            return Vec::new();
        }

        let known_ids: HashSet<u64> = known
            .keys()
            .filter_map(|s| s.parse().ok())
            .collect();

        println!("Downloading Tatoeba sentence links...");
        let links_bytes = reqwest::blocking::get(LINKS_URL)
            .expect("failed to fetch Tatoeba links")
            .bytes()
            .expect("failed to read Tatoeba links bytes");

        let bz = BzDecoder::new(Cursor::new(links_bytes));
        let mut archive = Archive::new(bz);

        // translation_ids: known japanese sentence id → set of translation sentence ids
        let mut translation_ids: HashMap<u64, Vec<u64>> = HashMap::new();

        'outer: for entry in archive.entries().expect("failed to read links tar") {
            let entry = entry.expect("failed to read tar entry");
            let path = entry.path().expect("failed to read tar entry path");
            if !path.to_string_lossy().contains("links") {
                continue;
            }

            let reader = BufReader::new(entry);
            for line in reader.lines() {
                let line = match line {
                    Ok(l) => l,
                    Err(_) => break 'outer,
                };
                let mut parts = line.splitn(2, '\t');
                let src: u64 = match parts.next().and_then(|s| s.parse().ok()) {
                    Some(n) => n,
                    None => continue,
                };
                let tgt: u64 = match parts.next().and_then(|s| s.parse().ok()) {
                    Some(n) => n,
                    None => continue,
                };
                if known_ids.contains(&src) {
                    translation_ids.entry(src).or_default().push(tgt);
                } else if known_ids.contains(&tgt) {
                    translation_ids.entry(tgt).or_default().push(src);
                }
            }
            break;
        }

        let target_ids: HashSet<u64> = translation_ids.values().flatten().copied().collect();
        if target_ids.is_empty() {
            println!("No Tatoeba translation links found for known sentences");
            return Vec::new();
        }

        // id_to_jpn_source: translation sentence id → example_id in our DB
        let mut id_to_example: HashMap<u64, i64> = HashMap::new();
        for (src_id, tgt_ids) in &translation_ids {
            if let Some(&example_id) = known.get(&src_id.to_string()) {
                for &tgt in tgt_ids {
                    id_to_example.insert(tgt, example_id);
                }
            }
        }

        let mut results: Vec<TatoebaTranslation> = Vec::new();

        for lang in langs {
            let url = SENTENCES_URL.replace("{lang}", lang);
            println!("Downloading Tatoeba {} sentences...", lang);

            let bytes = match reqwest::blocking::get(&url) {
                Ok(resp) if resp.status().is_success() => {
                    resp.bytes().expect("failed to read Tatoeba sentence bytes")
                }
                Ok(resp) => {
                    println!("  Skipping {} (HTTP {})", lang, resp.status());
                    continue;
                }
                Err(e) => {
                    println!("  Skipping {} ({})", lang, e);
                    continue;
                }
            };

            let reader = BufReader::new(BzDecoder::new(Cursor::new(bytes)));
            let mut count = 0usize;

            for line in reader.lines() {
                let line = line.expect("failed to read sentence line");
                let mut parts = line.splitn(3, '\t');
                let id: u64 = match parts.next().and_then(|s| s.parse().ok()) {
                    Some(n) => n,
                    None => continue,
                };
                let _lang_col = parts.next();
                let text = match parts.next() {
                    Some(t) => t.to_string(),
                    None => continue,
                };

                if let Some(&example_id) = id_to_example.get(&id) {
                    results.push(TatoebaTranslation {
                        example_id,
                        lang: lang.clone(),
                        text,
                    });
                    count += 1;
                }
            }
            println!("  Found {} {} translations", count, lang);
        }

        println!("Total Tatoeba translations: {}", results.len());
        results
    }
}
