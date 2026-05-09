use serde::Deserialize;

#[derive(Deserialize)]
pub struct EntryDto{
    #[serde(rename = "ent_seq")]
    pub id: u32,
    #[serde(rename = "k_ele", default)]
    pub kanji_readings: Vec<KanjiReadingDto>,
    #[serde(rename = "r_ele", default)]
    pub readings: Vec<ReadingDto>,
    #[serde(rename = "sense", default)]
    pub senses: Vec<SenseDto>
}

#[derive(Deserialize)]
pub struct KanjiReadingDto {
    #[serde(rename = "keb")]
    pub kanji: String,
    #[serde(rename = "ke_pri", default)]
    pub priority: Vec<String>,
    #[serde(rename = "ke_inf", default)]
    pub info: Vec<String>
}

#[derive(Deserialize)]
pub struct ReadingDto {
    #[serde(rename = "reb")]
    pub text: String,
    #[serde(rename = "re_restr", default)]
    pub restricted_readings: Vec<String>,
    #[serde(rename = "re_pri", default)]
    pub priority: Vec<String>,
    #[serde(rename = "re_inf", default)]
    pub info: Vec<String>,
    #[serde(rename = "re_nokanji", default)]
    pub no_kanji: Vec<NoKanjiDto>
}

#[derive(Deserialize)]
pub struct NoKanjiDto {   
}

#[derive(Deserialize)]
pub struct SenseDto {
    #[serde(rename = "pos", default)]
    pub pos: Vec<String>,
    #[serde(rename = "misc", default)]
    pub misc: Vec<String>,
    #[serde(rename = "xref", default)]
    pub refs: Vec<String>,
    #[serde(rename = "gloss", default)]
    pub glosses: Vec<GlossDto>,
    #[serde(rename = "s_inf", default)]
    pub info: Vec<String>,
    #[serde(rename = "dial", default)]
    pub dialects: Vec<String>,
    #[serde(rename = "example", default)]
    pub examples: Vec<ExampleDto>
}

fn default_lang() -> String {
    String::from("eng")
}

#[derive(Deserialize)]
pub struct GlossDto {
    #[serde(rename = "$value", default)]
    pub text: String,
    #[serde(rename = "@g_type", default)]
    pub type_: Option<String>,
    #[serde(rename = "@xml:lang", default = "default_lang")]
    pub lang: String,
}

#[derive(Deserialize)]
pub struct ExampleDto {
    #[serde(rename = "ex_srce")]
    pub source_: SourceDto,
    #[serde(rename = "ex_text")]
    pub text: String,
    #[serde(rename = "ex_sent", default)]
    pub sentences: Vec<SentenceDto>
}

#[derive(Deserialize)]
pub struct SentenceDto {
    #[serde(rename = "@xml:lang")]
    pub lang: String,
    #[serde(rename = "$value", default)]
    pub text: String
}

#[derive(Deserialize)]
pub struct SourceDto {
    #[serde(rename = "@exsrc_type")]
    pub name: String,
    #[serde(rename = "$value", default)]
    pub id: Option<String>
}