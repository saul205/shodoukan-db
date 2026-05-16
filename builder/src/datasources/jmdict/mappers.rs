use core::domain::models::entry::{CrossReference, Entry, Example, Gloss, KanjiReading, Reading, Sense, Source};
use crate::datasources::jmdict::dtos::{EntryDto, ExampleDto, GlossDto, KanjiReadingDto, ReadingDto, SenseDto, SourceDto};

pub struct JMDictMapper;

impl JMDictMapper {
    pub fn new() -> Self {
        Self
    }

    fn map_source(&self, dto: SourceDto) -> Source {
        Source { name: dto.name, id: dto.id }
    }

    fn map_gloss(&self, dto: GlossDto) -> Option<Gloss> {
        let text = dto.text?.trim().to_string();
        if text.is_empty() {
            return None;
        }
        Some(Gloss { text, type_: dto.type_, lang: Some(dto.lang) })
    }

    fn map_example(&self, dto: ExampleDto) -> Example {
        Example {
            source_: self.map_source(dto.source_),
            text: dto.text,
            sentences: dto.sentences.into_iter().map(|s| (s.lang, s.text)).collect(),
        }
    }

    fn map_reading(&self, dto: ReadingDto) -> Reading {
        Reading {
            text: dto.text,
            priority: dto.priority,
            no_kanji: !dto.no_kanji.is_empty(),
            info: dto.info,
        }
    }

    fn map_cross_reference(&self, dto: String) -> CrossReference {
        let parts: Vec<&str> = dto.split(|c| c == '.' || c == '・').collect();
        CrossReference {
            reference: parts[0].to_string(),
            reading: parts.get(1).map(|s| s.to_string()),
            sense_idx: parts.get(2).and_then(|s| s.parse().ok()),
        }
    }

    fn map_sense(&self, dto: SenseDto) -> Sense {
        Sense {
            pos: dto.pos,
            misc: dto.misc,
            refs: dto.refs.into_iter().map(|r| self.map_cross_reference(r)).collect(),
            glosses: dto.glosses.into_iter().filter_map(|g| self.map_gloss(g)).collect(),
            info: dto.info,
            dialects: dto.dialects,
            examples: dto.examples.into_iter().map(|e| self.map_example(e)).collect(),
        }
    }

    fn map_kanji_reading(&self, dto: KanjiReadingDto) -> KanjiReading {
        KanjiReading {
            kanji: dto.kanji,
            restricted_readings: Vec::new(),
            priority: dto.priority,
            info: dto.info,
        }
    }

    pub fn map_entry_to_domain(&self, dto: EntryDto) -> Entry {
        let mut kanji_readings: Vec<KanjiReading> = dto.kanji_readings
            .into_iter()
            .map(|kr| self.map_kanji_reading(kr))
            .collect();

        let mut readings: Vec<Reading> = Vec::new();

        for r_dto in dto.readings {
            let restrictions = r_dto.restricted_readings.clone();
            let r = self.map_reading(r_dto);

            for restriction in restrictions {
                for kr in &mut kanji_readings {
                    if kr.kanji == restriction {
                        kr.restricted_readings.push(r.clone());
                    }
                }
            }

            readings.push(r);
        }

        Entry {
            id: dto.id,
            jlpt: None,
            readings,
            kanji_readings,
            senses: dto.senses.into_iter().map(|s| self.map_sense(s)).collect(),
        }
    }
}
