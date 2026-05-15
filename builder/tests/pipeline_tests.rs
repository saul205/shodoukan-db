use std::fs::File;
use std::io::BufReader;

use builder::datasources::jmdict::parser::JMDictSource;
use builder::datasources::kanjidic::parser::KanjiDicSource;
use builder::traits::datasource::Datasource;
use core::infrastructure::sqlite::{connection, repository::{EntryRepository, KanjiRepository}};

const JMDICT_FIXTURE: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/jmdict_sample.xml");
const KANJIDIC_FIXTURE: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/kanjidic_sample.xml");

fn jmdict_source() -> JMDictSource {
    JMDictSource { ds_url: String::new() }
}

fn kanjidic_source() -> KanjiDicSource {
    KanjiDicSource { ds_url: String::new() }
}

fn parse_jmdict() -> Vec<core::domain::models::entry::Entry> {
    let reader = BufReader::new(File::open(JMDICT_FIXTURE).expect("jmdict fixture missing"));
    jmdict_source().parse(reader)
}

fn parse_kanjidic() -> Vec<core::domain::models::kanji::Kanji> {
    let reader = BufReader::new(File::open(KANJIDIC_FIXTURE).expect("kanjidic fixture missing"));
    kanjidic_source().parse(reader)
}

// ── JMDict pipeline ──────────────────────────────────────────────────────────

#[test]
fn jmdict_parses_all_entries() {
    let entries = parse_jmdict();
    assert_eq!(entries.len(), 17);
}

#[test]
fn jmdict_entry_without_kanji_has_no_kanji_readings() {
    let entries = parse_jmdict();
    let entry = entries.iter().find(|e| e.id == 1000000).expect("entry 1000000 missing");
    assert!(entry.kanji_readings.is_empty());
    assert_eq!(entry.readings[0].text, "ヽ");
}

#[test]
fn jmdict_entity_references_resolved_in_pos() {
    let entries = parse_jmdict();
    let entry = entries.iter().find(|e| e.id == 1000000).expect("entry 1000000 missing");
    assert_eq!(entry.senses[0].pos, vec!["unclassified"]);
}

#[test]
fn jmdict_gloss_type_preserved() {
    let entries = parse_jmdict();
    let entry = entries.iter().find(|e| e.id == 1000010).expect("entry 1000010 missing");
    let gloss = &entry.senses[0].glosses[0];
    assert_eq!(gloss.text, "voiced repetition mark in katakana");
    assert_eq!(gloss.type_.as_deref(), Some("expl"));
}

#[test]
fn jmdict_reading_restrictions_linked_to_kanji() {
    let entries = parse_jmdict();
    let entry = entries.iter().find(|e| e.id == 1000110).expect("entry 1000110 missing");

    let cd_player = entry.kanji_readings.iter()
        .find(|kr| kr.kanji == "ＣＤプレーヤー")
        .expect("ＣＤプレーヤー kanji reading missing");
    assert_eq!(cd_player.restricted_readings.len(), 1);
    assert_eq!(cd_player.restricted_readings[0].text, "シーディープレーヤー");
}

#[test]
fn jmdict_ke_inf_entity_resolved() {
    let entries = parse_jmdict();
    let entry = entries.iter().find(|e| e.id == 1000200).expect("entry 1000200 missing");
    let search_only = entry.kanji_readings.iter()
        .find(|kr| kr.kanji == "あ・うんの呼吸")
        .expect("search-only reading missing");
    assert!(search_only.info.contains(&"search-only kanji form".to_string()));
}

#[test]
fn jmdict_multiple_glosses_all_inserted() {
    let entries = parse_jmdict();
    let entry = entries.iter().find(|e| e.id == 1000220).expect("entry 1000220 missing");
    let eng_glosses: Vec<_> = entry.senses[0].glosses.iter()
        .filter(|g| g.lang.as_deref() == Some("eng"))
        .collect();
    assert!(eng_glosses.len() >= 7, "expected ≥7 English glosses, got {}", eng_glosses.len());
}

#[test]
fn jmdict_inserts_into_all_tables() {
    let entries = parse_jmdict();
    let conn = connection::open_in_memory().unwrap();
    let repo = EntryRepository::new(&conn);
    for entry in &entries {
        repo.insert(entry).unwrap();
    }

    let entry_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM entries", [], |r| r.get(0))
        .unwrap();
    assert_eq!(entry_count, 17);

    let reading_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM readings", [], |r| r.get(0))
        .unwrap();
    assert!(reading_count >= 15);

    let gloss_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM glosses", [], |r| r.get(0))
        .unwrap();
    assert!(gloss_count >= 15);
}

#[test]
fn jmdict_fts_indexes_english_glosses() {
    let entries = parse_jmdict();
    let conn = connection::open_in_memory().unwrap();
    let repo = EntryRepository::new(&conn);
    for entry in &entries {
        repo.insert(entry).unwrap();
    }

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM glosses_fts WHERE glosses_fts MATCH 'obvious'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn jmdict_entry_kanji_junction_populated_for_cjk_entries() {
    let entries = parse_jmdict();
    let conn = connection::open_in_memory().unwrap();
    let repo = EntryRepository::new(&conn);
    for entry in &entries {
        repo.insert(entry).unwrap();
    }

    // 明白 contains 明 (U+660E) and 白 (U+767D), both CJK Unified Ideographs
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM entry_kanji WHERE entry_id = 1000220",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 2);

    let literals: Vec<String> = {
        let mut stmt = conn
            .prepare("SELECT literal FROM entry_kanji WHERE entry_id = 1000220 ORDER BY literal")
            .unwrap();
        stmt.query_map([], |r| r.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect()
    };
    assert!(literals.contains(&"明".to_string()));
    assert!(literals.contains(&"白".to_string()));
}

#[test]
fn jmdict_reading_restrictions_stored_in_db() {
    let entries = parse_jmdict();
    let conn = connection::open_in_memory().unwrap();
    let repo = EntryRepository::new(&conn);
    for entry in &entries {
        repo.insert(entry).unwrap();
    }

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM reading_restrictions", [], |r| r.get(0))
        .unwrap();
    assert!(count >= 2, "expected ≥2 reading restrictions (CD player entry), got {count}");
}

// ── KANJIDIC pipeline ────────────────────────────────────────────────────────

#[test]
fn kanjidic_parses_all_characters() {
    let kanjis = parse_kanjidic();
    assert_eq!(kanjis.len(), 20);
}

#[test]
fn kanjidic_fields_parsed_correctly() {
    let kanjis = parse_kanjidic();
    let azia = kanjis.iter().find(|k| k.literal == "亜").expect("亜 missing");
    assert_eq!(azia.grade, Some(8));
    assert_eq!(azia.stroke_count, 7);
    assert_eq!(azia.freq, Some(1509));
    assert_eq!(azia.jlpt, Some(1));
}

#[test]
fn kanjidic_on_kun_readings_filtered() {
    let kanjis = parse_kanjidic();
    let azia = kanjis.iter().find(|k| k.literal == "亜").expect("亜 missing");
    assert_eq!(azia.on_readings, vec!["ア"]);
    assert_eq!(azia.kun_readings, vec!["つ.ぐ"]);
}

#[test]
fn kanjidic_nanori_parsed() {
    let kanjis = parse_kanjidic();
    let azia = kanjis.iter().find(|k| k.literal == "亜").expect("亜 missing");
    assert!(azia.nanori.contains(&"つぎ".to_string()));
    assert!(azia.nanori.contains(&"つぐ".to_string()));
}

#[test]
fn kanjidic_meanings_include_english_and_other_langs() {
    let kanjis = parse_kanjidic();
    let azia = kanjis.iter().find(|k| k.literal == "亜").expect("亜 missing");
    let english: Vec<_> = azia.meanings.iter().filter(|m| m.lang == "en").collect();
    assert!(english.iter().any(|m| m.text == "Asia"));

    let french: Vec<_> = azia.meanings.iter().filter(|m| m.lang == "fr").collect();
    assert!(!french.is_empty());
}

#[test]
fn kanjidic_multiple_stroke_counts_takes_first() {
    let kanjis = parse_kanjidic();
    // 逢 has stroke_count [10, 9, 11] in the fixture; first wins
    let au = kanjis.iter().find(|k| k.literal == "逢").expect("逢 missing");
    assert_eq!(au.stroke_count, 10);
}

#[test]
fn kanjidic_character_without_jlpt_has_none() {
    let kanjis = parse_kanjidic();
    // 唖 has no <jlpt> in the fixture
    let a = kanjis.iter().find(|k| k.literal == "唖").expect("唖 missing");
    assert_eq!(a.jlpt, None);
}

#[test]
fn kanjidic_inserts_into_db() {
    let kanjis = parse_kanjidic();
    let conn = connection::open_in_memory().unwrap();
    let repo = KanjiRepository::new(&conn);
    for kanji in &kanjis {
        repo.insert(kanji).unwrap();
    }

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM kanji", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 20);

    let meaning_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM kanji_meanings", [], |r| r.get(0))
        .unwrap();
    assert!(meaning_count >= 20);
}

#[test]
fn kanjidic_fts_indexes_meanings() {
    let kanjis = parse_kanjidic();
    let conn = connection::open_in_memory().unwrap();
    let repo = KanjiRepository::new(&conn);
    for kanji in &kanjis {
        repo.insert(kanji).unwrap();
    }

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM kanji_meanings_fts WHERE kanji_meanings_fts MATCH 'love'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    // 愛 has meaning "love"
    assert_eq!(count, 1);
}

#[test]
fn kanjidic_readings_stored_as_json_arrays() {
    let kanjis = parse_kanjidic();
    let conn = connection::open_in_memory().unwrap();
    let repo = KanjiRepository::new(&conn);
    for kanji in &kanjis {
        repo.insert(kanji).unwrap();
    }

    let on: String = conn
        .query_row(
            "SELECT on_readings FROM kanji WHERE literal = '亜'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(on, r#"["ア"]"#);
}

// ── Reference entity: all DB fields ──────────────────────────────────────────

fn populated_db() -> rusqlite::Connection {
    let conn = connection::open_in_memory().unwrap();
    let kanji_repo = KanjiRepository::new(&conn);
    for k in &parse_kanjidic() {
        kanji_repo.insert(k).unwrap();
    }
    let entry_repo = EntryRepository::new(&conn);
    for e in &parse_jmdict() {
        entry_repo.insert(e).unwrap();
    }
    conn
}

#[test]
fn jmdict_reference_entry_all_db_fields() {
    let conn = populated_db();

    // entries
    let jlpt: Option<i64> = conn
        .query_row("SELECT jlpt FROM entries WHERE id = 1000220", [], |r| r.get(0))
        .unwrap();
    assert_eq!(jlpt, None);

    // kanji_readings
    let (kanji, priority, info): (String, String, String) = conn
        .query_row(
            "SELECT kanji, priority, info FROM kanji_readings WHERE entry_id = 1000220",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    assert_eq!(kanji, "明白");
    assert_eq!(priority, r#"["ichi1","news1","nf10"]"#);
    assert_eq!(info, "[]");

    // readings
    let (text, no_kanji, r_priority, r_info): (String, i64, String, String) = conn
        .query_row(
            "SELECT text, no_kanji, priority, info FROM readings WHERE entry_id = 1000220",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .unwrap();
    assert_eq!(text, "めいはく");
    assert_eq!(no_kanji, 0);
    assert_eq!(r_priority, r#"["ichi1","news1","nf10"]"#);
    assert_eq!(r_info, "[]");

    // senses (first sense)
    let (pos, misc, dialects, s_info): (String, String, String, String) = conn
        .query_row(
            "SELECT pos, misc, dialects, info FROM senses WHERE entry_id = 1000220 LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .unwrap();
    assert_eq!(pos, r#"["adjectival nouns or quasi-adjectives (keiyodoshi)"]"#);
    assert_eq!(misc, "[]");
    assert_eq!(dialects, "[]");
    assert_eq!(s_info, "[]");

    // glosses (English, first sense)
    let eng_glosses: Vec<String> = {
        let mut stmt = conn
            .prepare(
                "SELECT g.text FROM glosses g
                 JOIN senses s ON s.id = g.sense_id
                 WHERE s.entry_id = 1000220 AND g.lang = 'eng'
                 ORDER BY g.id",
            )
            .unwrap();
        stmt.query_map([], |r| r.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect()
    };
    assert_eq!(
        eng_glosses,
        ["obvious", "clear", "plain", "evident", "apparent", "explicit", "overt"]
    );

    // entry_kanji
    let mut literals: Vec<String> = {
        let mut stmt = conn
            .prepare("SELECT literal FROM entry_kanji WHERE entry_id = 1000220 ORDER BY literal")
            .unwrap();
        stmt.query_map([], |r| r.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect()
    };
    literals.sort();
    assert_eq!(literals, ["明", "白"]);
}

#[test]
fn kanjidic_reference_kanji_all_db_fields() {
    let conn = populated_db();

    // kanji main fields
    let (grade, stroke_count, freq, jlpt, on_readings, kun_readings, nanori): (
        i64, i64, i64, i64, String, String, String,
    ) = conn
        .query_row(
            "SELECT grade, stroke_count, freq, jlpt, on_readings, kun_readings, nanori
             FROM kanji WHERE literal = '亜'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?)),
        )
        .unwrap();
    assert_eq!(grade, 8);
    assert_eq!(stroke_count, 7);
    assert_eq!(freq, 1509);
    assert_eq!(jlpt, 1);
    assert_eq!(on_readings, r#"["ア"]"#);
    assert_eq!(kun_readings, r#"["つ.ぐ"]"#);
    assert_eq!(nanori, r#"["や","つぎ","つぐ"]"#);

    // English meanings
    let en_meanings: Vec<String> = {
        let mut stmt = conn
            .prepare(
                "SELECT text FROM kanji_meanings WHERE literal = '亜' AND lang = 'en' ORDER BY id",
            )
            .unwrap();
        stmt.query_map([], |r| r.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect()
    };
    assert_eq!(en_meanings, ["Asia", "rank next", "come after", "-ous"]);

    // French meanings
    let fr_meanings: Vec<String> = {
        let mut stmt = conn
            .prepare(
                "SELECT text FROM kanji_meanings WHERE literal = '亜' AND lang = 'fr' ORDER BY id",
            )
            .unwrap();
        stmt.query_map([], |r| r.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect()
    };
    assert_eq!(fr_meanings, ["Asie", "suivant", "sub-", "sous-"]);
}
