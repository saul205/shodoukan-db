use core::domain::models::entry::{CrossReference, Entry, Example, Gloss, KanjiReading, Reading, Sense, Source};
use crate::datasources::jmdict::dtos::{EntryDto, ExampleDto, GlossDto, KanjiReadingDto, ReadingDto, SenseDto, SourceDto};
use std::collections::HashMap;


pub struct JMDictMapper {
    pub entities: HashMap<String, String>,
}

impl JMDictMapper {
    pub fn new(entities: HashMap<String, String>) -> Self {
        Self { entities }
    }

    fn map_sources_to_domain(&self, dto: SourceDto) -> Source {
        Source {
            name: dto.name,
            id: dto.id
        }
    }

    fn map_glosses_to_domain(&self, dto: GlossDto) -> Gloss {
        Gloss {
            text: dto.text,
            type_: dto.type_,
            lang: Some(String::from("en"))
        }
    }

    fn map_examples_to_domain(&self, dto: ExampleDto) -> Example {
        Example {
            source_: self.map_sources_to_domain(dto.source_),
            text: dto.text,
            sentences: dto.sentences.into_iter().map(|s| (s.lang, s.text)).collect()
        }
    }

    fn map_readings_to_domain(&self, dto: ReadingDto) -> Reading {
        Reading {
            text: dto.text,
            priority: dto.priority,
            no_kanji: !dto.no_kanji.is_empty(),
            info: dto.info.into_iter().map(|i| self.resolve(i)).collect(),
        }
    }

    fn map_cross_reference_to_domain(&self, dto: String) -> CrossReference {
    let parts: Vec<&str> = dto.split(|c| c == '.' || c == '・').collect();
        let reference = parts[0].to_string();
        let reading = if parts.len() > 1 { Some(parts[1].to_string()) } else { None };
        let sense_idx = if parts.len() > 2 { parts[2].parse::<usize>().ok() } else { None };    
        CrossReference {
            reference,
            reading,
            sense_idx
        }
    }

    fn map_senses_to_domain(&self, dto: SenseDto) -> Sense {

        let pos: Vec<String> = dto.pos.into_iter().map(|p| self.resolve(p)).collect();
        Sense{
            pos: pos,
            misc: dto.misc.into_iter().map(|m| self.resolve(m)).collect(),
            refs: dto.refs.into_iter().map(|r| self.map_cross_reference_to_domain(r)).collect(),
            glosses: dto.glosses.into_iter().map(|g| self.map_glosses_to_domain(g)).collect(),
            info: dto.info,
            dialects: dto.dialects,
            examples: dto.examples.into_iter().map(|e| self.map_examples_to_domain(e)).collect()
        }
    }

    fn map_kreadings_to_domain(&self, dto: KanjiReadingDto) -> KanjiReading {
        KanjiReading {
            kanji: dto.kanji,
            restricted_readings: Vec::new(),
            priority: dto.priority,
            info: dto.info.into_iter().map(|i| self.resolve(i)).collect(),
        }
    }
    
    pub fn map_entry_to_domain(&self, dto: EntryDto) -> Entry {

        let mut readings: Vec<Reading> = Vec::new();
        let mut kanji_readings: Vec<KanjiReading> = Vec::new();

        for kr_dto in dto.kanji_readings {
            let kr = self.map_kreadings_to_domain(kr_dto);
            kanji_readings.push(kr);
        }

        for r_dto in dto.readings {
            let restrictions = r_dto.restricted_readings.clone();
            let r = self.map_readings_to_domain(r_dto);

            // Restricted readings belong to a kanji reading
            // Iterate over kanji readings
            for rr in restrictions{
                // Iterate over restricted readings
                for kr in &mut kanji_readings{
                    // If the kanji matches the restricted reading, add it to the kanji reading
                    if kr.kanji == rr {
                        kr.restricted_readings.push(r.clone());
                    }
                }   
            }
            
            readings.push(r);
        }

        // Mapping logic here, utilizing self.entities as needed
        Entry {
            id: dto.id,
            readings,
            kanji_readings,
            senses: dto.senses.into_iter().map(|s| self.map_senses_to_domain(s)).collect()
        }
    }

    fn resolve(&self, code: String) -> String {
        let key = code.trim_matches(|c| c == '&' || c == ';');
        println!("{}", self.entities.get(key).cloned().unwrap_or_else(|| code.to_string()));
        self.entities.get(key).cloned().unwrap_or_else(|| code.to_string())
    }
}