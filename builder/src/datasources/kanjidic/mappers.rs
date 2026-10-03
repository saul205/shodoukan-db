use core::domain::models::kanji::{Kanji, Meaning};
use crate::datasources::kanjidic::dtos::CharacterDto;

pub struct KanjiDicMapper;

impl KanjiDicMapper {
    pub fn new() -> Self {
        Self
    }

    pub fn map_character_to_domain(&self, dto: CharacterDto) -> Kanji {
        let (on_readings, kun_readings, meanings, nanori) = dto
            .reading_meaning
            .map(|rm| {
                let mut on = Vec::new();
                let mut kun = Vec::new();
                let mut meanings = Vec::new();

                for group in rm.rmgroups {
                    for r in group.readings {
                        match r.r_type.as_str() {
                            "ja_on" => on.push(r.text),
                            "ja_kun" => kun.push(r.text),
                            _ => {}
                        }
                    }
                    for m in group.meanings {
                        meanings.push(Meaning { text: m.text, lang: m.lang });
                    }
                }

                (on, kun, meanings, rm.nanori)
            })
            .unwrap_or_default();

        Kanji {
            literal: dto.literal,
            grade: dto.misc.grade,
            stroke_count: dto.misc.stroke_count.into_iter().next().unwrap_or(0),
            freq: dto.misc.freq,
            jlpt: dto.misc.jlpt.and_then(old_jlpt_to_n_level),
            on_readings,
            kun_readings,
            meanings,
            nanori,
        }
    }
}

/// KANJIDIC2 uses the pre-2010 4-level JLPT scale; convert it to the N1–N5 scale.
/// Only a fallback: step 8 overwrites it for every kanji in the JLPT list.
fn old_jlpt_to_n_level(level: u8) -> Option<u8> {
    match level {
        1 => Some(1),
        2 => Some(2),
        3 => Some(4),
        4 => Some(5),
        _ => None,
    }
}
