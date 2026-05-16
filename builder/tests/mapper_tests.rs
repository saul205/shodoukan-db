use builder::datasources::jmdict::{
    dtos::{EntryDto, GlossDto, KanjiReadingDto, NoKanjiDto, ReadingDto, SenseDto},
    mappers::JMDictMapper,
};

fn mapper() -> JMDictMapper {
    JMDictMapper::new()
}

fn reading(text: &str) -> ReadingDto {
    ReadingDto {
        text: String::from(text),
        restricted_readings: vec![],
        priority: vec![],
        info: vec![],
        no_kanji: vec![],
    }
}

#[test]
fn maps_basic_fields() {
    let dto = EntryDto {
        id: 1000001,
        kanji_readings: vec![KanjiReadingDto {
            kanji: String::from("食べる"),
            priority: vec![String::from("ichi1")],
            info: vec![],
        }],
        readings: vec![reading("たべる")],
        senses: vec![SenseDto {
            pos: vec![],
            misc: vec![],
            refs: vec![],
            glosses: vec![GlossDto {
                text: Some(String::from("to eat")),
                type_: None,
                lang: String::from("eng"),
            }],
            info: vec![],
            dialects: vec![],
            examples: vec![],
        }],
    };

    let entry = mapper().map_entry_to_domain(dto);

    assert_eq!(entry.id, 1000001);
    assert_eq!(entry.kanji_readings[0].kanji, "食べる");
    assert_eq!(entry.kanji_readings[0].priority, vec!["ichi1"]);
    assert_eq!(entry.readings[0].text, "たべる");
    assert_eq!(entry.senses[0].glosses[0].text, "to eat");
    assert_eq!(entry.senses[0].glosses[0].lang.as_deref(), Some("eng"));
}

#[test]
fn preserves_gloss_language() {
    let entry = mapper().map_entry_to_domain(EntryDto {
        id: 1,
        kanji_readings: vec![],
        readings: vec![reading("ヽ")],
        senses: vec![SenseDto {
            pos: vec![],
            misc: vec![],
            refs: vec![],
            glosses: vec![
                GlossDto { text: Some(String::from("repetition mark")), type_: None, lang: String::from("eng") },
                GlossDto { text: Some(String::from("hitotsuten")), type_: None, lang: String::from("dut") },
            ],
            info: vec![],
            dialects: vec![],
            examples: vec![],
        }],
    });

    assert_eq!(entry.senses[0].glosses[0].lang.as_deref(), Some("eng"));
    assert_eq!(entry.senses[0].glosses[1].lang.as_deref(), Some("dut"));
}

#[test]
fn maps_no_kanji_flag() {
    let entry = mapper().map_entry_to_domain(EntryDto {
        id: 1,
        kanji_readings: vec![],
        readings: vec![ReadingDto {
            text: String::from("ドア"),
            restricted_readings: vec![],
            priority: vec![],
            info: vec![],
            no_kanji: vec![NoKanjiDto {}],
        }],
        senses: vec![],
    });

    assert!(entry.readings[0].no_kanji);
}

#[test]
fn maps_restricted_readings_to_kanji_reading() {
    let entry = mapper().map_entry_to_domain(EntryDto {
        id: 1,
        kanji_readings: vec![KanjiReadingDto {
            kanji: String::from("食べ物"),
            priority: vec![],
            info: vec![],
        }],
        readings: vec![ReadingDto {
            text: String::from("たべもの"),
            restricted_readings: vec![String::from("食べ物")],
            priority: vec![],
            info: vec![],
            no_kanji: vec![],
        }],
        senses: vec![],
    });

    assert_eq!(entry.kanji_readings[0].restricted_readings.len(), 1);
    assert_eq!(entry.kanji_readings[0].restricted_readings[0].text, "たべもの");
    assert_eq!(entry.readings[0].text, "たべもの");
}

#[test]
fn maps_cross_reference_with_reading_and_sense_idx() {
    let entry = mapper().map_entry_to_domain(EntryDto {
        id: 1,
        kanji_readings: vec![],
        readings: vec![],
        senses: vec![SenseDto {
            pos: vec![],
            misc: vec![],
            refs: vec![String::from("食べる・たべる・1")],
            glosses: vec![],
            info: vec![],
            dialects: vec![],
            examples: vec![],
        }],
    });

    let xref = &entry.senses[0].refs[0];
    assert_eq!(xref.reference, "食べる");
    assert_eq!(xref.reading.as_deref(), Some("たべる"));
    assert_eq!(xref.sense_idx, Some(1));
}

#[test]
fn maps_cross_reference_without_reading() {
    let entry = mapper().map_entry_to_domain(EntryDto {
        id: 1,
        kanji_readings: vec![],
        readings: vec![],
        senses: vec![SenseDto {
            pos: vec![],
            misc: vec![],
            refs: vec![String::from("食べる")],
            glosses: vec![],
            info: vec![],
            dialects: vec![],
            examples: vec![],
        }],
    });

    let xref = &entry.senses[0].refs[0];
    assert_eq!(xref.reference, "食べる");
    assert!(xref.reading.is_none());
    assert!(xref.sense_idx.is_none());
}
