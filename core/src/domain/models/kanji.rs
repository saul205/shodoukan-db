pub struct Kanji {
    pub literal: String,
    pub grade: Option<u8>,
    pub stroke_count: u8,
    pub freq: Option<u16>,
    pub jlpt: Option<u8>,
    pub on_readings: Vec<String>,
    pub kun_readings: Vec<String>,
    pub meanings: Vec<Meaning>,
    pub nanori: Vec<String>,
}

pub struct Meaning {
    pub text: String,
    pub lang: String,
}
