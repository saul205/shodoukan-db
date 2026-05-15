use serde::Deserialize;

#[derive(Deserialize)]
pub struct VocabItemDto {
    pub reading: String,
    pub level: u8,
}

pub struct JlptVocabEntry {
    pub key: String,
    pub reading: String,
    pub level: u8,
}

pub struct JlptKanjiEntry {
    pub literal: String,
    pub level: u8,
}
