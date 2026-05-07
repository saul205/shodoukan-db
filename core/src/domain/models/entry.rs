use std::collections::HashMap;

#[derive(Debug)]
pub struct Entry{
    pub id: u32,
    pub kanji_readings: Vec<KanjiReading>,
    pub readings: Vec<Reading>,
    pub senses: Vec<Sense>
}

#[derive(Debug)]
pub struct KanjiReading {
    pub kanji: String,
    pub restricted_readings: Vec<Reading>,
    pub priority: Vec<String>,
    pub info: Vec<String>
}

#[derive(Clone, Debug)]
pub struct Reading {
    pub text: String,
    pub priority: Vec<String>,
    pub no_kanji: bool,
    pub info: Vec<String>
}

#[derive(Debug)]
pub struct Sense {
    pub pos: Vec<String>,
    pub misc: Vec<String>,
    pub refs: Vec<CrossReference>,
    pub glosses: Vec<Gloss>,
    pub info: Vec<String>,
    pub dialects: Vec<String>,
    pub examples: Vec<Example>
}

#[derive(Debug)]
pub struct CrossReference {
    pub reference: String,
    pub reading: Option<String>,
    pub sense_idx: Option<usize>
}

#[derive(Debug)]
pub struct Gloss {
    pub text: String,
    pub type_: Option<String>,
    pub lang: Option<String>
}

#[derive(Debug)]
pub struct Example {
    pub source_: Source,
    pub text: String,
    pub sentences: HashMap<String, String>
}

#[derive(Debug)]
pub struct Source {
    pub name: String,
    pub id: Option<String>
}