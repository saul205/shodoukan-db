use builder::datasources::kanjidic::{
    dtos::{CharacterDto, MeaningDto, MiscDto, ReadingDto, ReadingMeaningDto, RmGroupDto},
    mappers::KanjiDicMapper,
};

fn mapper() -> KanjiDicMapper {
    KanjiDicMapper::new()
}

fn misc(grade: Option<u8>, stroke_count: u8, freq: Option<u16>, jlpt: Option<u8>) -> MiscDto {
    MiscDto {
        grade,
        stroke_count: vec![stroke_count],
        freq,
        jlpt,
    }
}

#[test]
fn maps_basic_fields() {
    let dto = CharacterDto {
        literal: String::from("本"),
        misc: misc(Some(1), 5, Some(10), Some(4)),
        reading_meaning: None,
    };

    let kanji = mapper().map_character_to_domain(dto);

    assert_eq!(kanji.literal, "本");
    assert_eq!(kanji.grade, Some(1));
    assert_eq!(kanji.stroke_count, 5);
    assert_eq!(kanji.freq, Some(10));
    assert_eq!(kanji.jlpt, Some(4));
}

#[test]
fn maps_on_and_kun_readings() {
    let dto = CharacterDto {
        literal: String::from("本"),
        misc: misc(Some(1), 5, None, None),
        reading_meaning: Some(ReadingMeaningDto {
            rmgroups: vec![RmGroupDto {
                readings: vec![
                    ReadingDto { r_type: String::from("ja_on"), text: String::from("ホン") },
                    ReadingDto { r_type: String::from("ja_kun"), text: String::from("もと") },
                    ReadingDto { r_type: String::from("pinyin"), text: String::from("ben3") },
                ],
                meanings: vec![],
            }],
            nanori: vec![],
        }),
    };

    let kanji = mapper().map_character_to_domain(dto);

    assert_eq!(kanji.on_readings, vec!["ホン"]);
    assert_eq!(kanji.kun_readings, vec!["もと"]);
}

#[test]
fn maps_english_meanings_with_default_lang() {
    let dto = CharacterDto {
        literal: String::from("本"),
        misc: misc(None, 5, None, None),
        reading_meaning: Some(ReadingMeaningDto {
            rmgroups: vec![RmGroupDto {
                readings: vec![],
                meanings: vec![
                    MeaningDto { lang: String::from("en"), text: String::from("book") },
                    MeaningDto { lang: String::from("en"), text: String::from("main") },
                    MeaningDto { lang: String::from("fr"), text: String::from("livre") },
                ],
            }],
            nanori: vec![],
        }),
    };

    let kanji = mapper().map_character_to_domain(dto);

    assert_eq!(kanji.meanings.len(), 3);
    assert_eq!(kanji.meanings[0].text, "book");
    assert_eq!(kanji.meanings[0].lang, "en");
    assert_eq!(kanji.meanings[2].lang, "fr");
}

#[test]
fn maps_nanori() {
    let dto = CharacterDto {
        literal: String::from("本"),
        misc: misc(None, 5, None, None),
        reading_meaning: Some(ReadingMeaningDto {
            rmgroups: vec![],
            nanori: vec![String::from("まと")],
        }),
    };

    let kanji = mapper().map_character_to_domain(dto);

    assert_eq!(kanji.nanori, vec!["まと"]);
}

#[test]
fn handles_missing_reading_meaning() {
    let dto = CharacterDto {
        literal: String::from("〃"),
        misc: misc(None, 2, None, None),
        reading_meaning: None,
    };

    let kanji = mapper().map_character_to_domain(dto);

    assert!(kanji.on_readings.is_empty());
    assert!(kanji.kun_readings.is_empty());
    assert!(kanji.meanings.is_empty());
    assert!(kanji.nanori.is_empty());
}

#[test]
fn takes_first_stroke_count_as_canonical() {
    let dto = CharacterDto {
        literal: String::from("飛"),
        misc: MiscDto {
            grade: None,
            stroke_count: vec![9, 10],
            freq: None,
            jlpt: None,
        },
        reading_meaning: None,
    };

    let kanji = mapper().map_character_to_domain(dto);

    assert_eq!(kanji.stroke_count, 9);
}
