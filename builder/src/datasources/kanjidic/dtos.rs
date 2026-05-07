use serde::Deserialize;

#[derive(Deserialize)]
pub struct CharacterDto {
    pub literal: String,
    pub misc: MiscDto,
    #[serde(rename = "reading_meaning")]
    pub reading_meaning: Option<ReadingMeaningDto>,
}

#[derive(Deserialize)]
pub struct MiscDto {
    pub grade: Option<u8>,
    #[serde(rename = "stroke_count", default)]
    pub stroke_count: Vec<u8>,
    pub freq: Option<u16>,
    pub jlpt: Option<u8>,
}

#[derive(Deserialize)]
pub struct ReadingMeaningDto {
    #[serde(rename = "rmgroup", default)]
    pub rmgroups: Vec<RmGroupDto>,
    #[serde(rename = "nanori", default)]
    pub nanori: Vec<String>,
}

#[derive(Deserialize)]
pub struct RmGroupDto {
    #[serde(rename = "reading", default)]
    pub readings: Vec<ReadingDto>,
    #[serde(rename = "meaning", default)]
    pub meanings: Vec<MeaningDto>,
}

#[derive(Deserialize)]
pub struct ReadingDto {
    #[serde(rename = "@r_type")]
    pub r_type: String,
    #[serde(rename = "$value")]
    pub text: String,
}

fn default_lang() -> String {
    String::from("en")
}

#[derive(Deserialize)]
pub struct MeaningDto {
    #[serde(rename = "@m_lang", default = "default_lang")]
    pub lang: String,
    #[serde(rename = "$value")]
    pub text: String,
}
