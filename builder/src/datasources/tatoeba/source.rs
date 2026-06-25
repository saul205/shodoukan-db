use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader, Cursor};
use bzip2::read::BzDecoder;
use core::infrastructure::sqlite::repository::TatoebaTranslation;
use tar::Archive;

const LINKS_URL: &str = "https://downloads.tatoeba.org/exports/links.tar.bz2";
const SENTENCES_URL: &str =
    "https://downloads.tatoeba.org/exports/per_language/{lang}/{lang}_sentences.tsv.bz2";

/// Decompress one pass over a `links.tar.bz2` buffer and call `f(src, tgt)` for every pair.
fn stream_links(bytes: &[u8], mut f: impl FnMut(u64, u64)) {
    let mut archive = Archive::new(BzDecoder::new(Cursor::new(bytes)));
    for entry in archive.entries().expect("failed to read links tar") {
        let entry = entry.expect("failed to read tar entry");
        if !entry.path().expect("failed to read path").to_string_lossy().contains("links") {
            continue;
        }
        let reader = BufReader::new(entry);
        for line in reader.lines() {
            let line = match line { Ok(l) => l, Err(_) => break };
            let mut parts = line.splitn(2, '\t');
            let (Some(src), Some(tgt)) = (
                parts.next().and_then(|s| s.parse::<u64>().ok()),
                parts.next().and_then(|s| s.parse::<u64>().ok()),
            ) else { continue };
            f(src, tgt);
        }
        break;
    }
}

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
        let links_bytes = crate::http::fetch_bytes(LINKS_URL);

        // ── Pass 1: direct translations of our known Japanese sentence IDs ──
        //
        // Streams through the compressed bytes without storing the decompressed content.
        // hop1_to_example: directly-linked sentence id → example_id in our DB

        let mut hop1_to_example: HashMap<u64, i64> = HashMap::new();
        stream_links(&links_bytes, |src, tgt| {
            if let Some(&eid) = known.get(&src.to_string()) {
                hop1_to_example.entry(tgt).or_insert(eid);
            } else if let Some(&eid) = known.get(&tgt.to_string()) {
                hop1_to_example.entry(src).or_insert(eid);
            }
        });
        println!("  {} direct translation links found", hop1_to_example.len());

        if hop1_to_example.is_empty() {
            println!("No Tatoeba translation links found for known sentences");
            return Vec::new();
        }

        // ── Pass 2: translations of translations (2-hop) ──
        //
        // A second BzDecoder over the same compressed bytes — no re-download.
        // id_to_example: all reachable sentence ids (hop1 + hop2) → example_id

        let mut id_to_example: HashMap<u64, i64> = hop1_to_example.clone();
        stream_links(&links_bytes, |src, tgt| {
            if let Some(&eid) = hop1_to_example.get(&src) {
                if !known_ids.contains(&tgt) { id_to_example.entry(tgt).or_insert(eid); }
            } else if let Some(&eid) = hop1_to_example.get(&tgt) {
                if !known_ids.contains(&src) { id_to_example.entry(src).or_insert(eid); }
            }
        });

        let hop2_count = id_to_example.len() - hop1_to_example.len();
        println!("  {} additional 2-hop translation links found", hop2_count);

        // ── Per-language sentence files ──

        let mut results: Vec<TatoebaTranslation> = Vec::new();

        for lang in langs {
            let url = SENTENCES_URL.replace("{lang}", crate::lang::to_tatoeba(lang));
            println!("Downloading Tatoeba {} sentences...", lang);

            let bytes = match crate::http::try_fetch_bytes(&url) {
                Some(b) => b,
                None => {
                    println!("  Skipping {} (not available)", lang);
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
                    results.push(TatoebaTranslation { example_id, lang: lang.clone(), text });
                    count += 1;
                }
            }
            println!("  Found {} {} translations", count, lang);
        }

        println!("Total Tatoeba translations: {}", results.len());
        results
    }
}
