use std::collections::HashMap;

use super::dtos::{JlptKanjiEntry, JlptVocabEntry, VocabItemDto};

const VOCAB_URL: &str =
    "https://github.com/Bluskyo/JLPT_Vocabulary/releases/latest/download/JLPT_vocab_ALL.json";
const KANJI_URL: &str =
    "https://github.com/Bluskyo/JLPT_Vocabulary/releases/latest/download/JLPT_kanji_ALL.json";

pub struct JlptSource;

impl JlptSource {
    pub fn fetch_vocab(&self) -> Vec<JlptVocabEntry> {
        println!("Downloading JLPT vocabulary...");
        let bytes = crate::http::fetch_bytes(VOCAB_URL);
        let map: HashMap<String, Vec<VocabItemDto>> =
            serde_json::from_slice(&bytes).expect("failed to parse JLPT vocab JSON");

        let entries: Vec<JlptVocabEntry> = map
            .into_iter()
            .flat_map(|(key, items): (String, Vec<VocabItemDto>)| {
                items.into_iter().map(move |item| JlptVocabEntry {
                    key: key.clone(),
                    reading: item.reading,
                    level: item.level,
                })
            })
            .collect();

        println!("Parsed {} JLPT vocab entries", entries.len());
        entries
    }

    pub fn fetch_kanji(&self) -> Vec<JlptKanjiEntry> {
        println!("Downloading JLPT kanji...");
        let bytes = crate::http::fetch_bytes(KANJI_URL);
        let map: HashMap<String, u8> =
            serde_json::from_slice(&bytes).expect("failed to parse JLPT kanji JSON");

        let entries: Vec<JlptKanjiEntry> = map
            .into_iter()
            .map(|(literal, level)| JlptKanjiEntry { literal, level })
            .collect();

        println!("Parsed {} JLPT kanji entries", entries.len());
        entries
    }
}
