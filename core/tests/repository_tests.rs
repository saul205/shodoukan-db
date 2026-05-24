use std::collections::HashMap;

use core::domain::models::entry::{
    CrossReference, Entry, Example, Gloss, KanjiReading, Reading, Sense, Source,
};
use core::infrastructure::sqlite::{connection, repository::{EntryRepository, KanjiRepository}};
use core::domain::models::kanji::Kanji;

fn sample_entry() -> Entry {
    Entry {
        id: 1000001,
        jlpt: None,
        kanji_readings: vec![KanjiReading {
            kanji: String::from("食べ物"),
            restricted_readings: vec![],
            priority: vec![String::from("ichi1")],
            info: vec![],
        }],
        readings: vec![Reading {
            text: String::from("たべもの"),
            priority: vec![String::from("ichi1")],
            no_kanji: false,
            info: vec![],
        }],
        senses: vec![Sense {
            pos: vec![String::from("noun")],
            misc: vec![],
            refs: vec![],
            glosses: vec![Gloss {
                text: String::from("food"),
                type_: None,
                lang: Some(String::from("en")),
            }],
            info: vec![],
            dialects: vec![],
            examples: vec![],
        }],
    }
}

#[test]
fn insert_populates_all_tables() {
    let conn = connection::open_in_memory().unwrap();
    let repo = EntryRepository::new(&conn);

    repo.insert(&sample_entry()).unwrap();

    let entry_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM entries", [], |r| r.get(0))
        .unwrap();
    assert_eq!(entry_count, 1);

    let kanji: String = conn
        .query_row(
            "SELECT kanji FROM kanji_readings WHERE entry_id = 1000001",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(kanji, "食べ物");

    let reading: String = conn
        .query_row(
            "SELECT text FROM readings WHERE entry_id = 1000001",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(reading, "たべもの");

    let gloss: String = conn
        .query_row(
            "SELECT g.text FROM glosses g
             JOIN senses s ON s.id = g.sense_id
             WHERE s.entry_id = 1000001",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(gloss, "food");

    let pos: String = conn
        .query_row("SELECT pos FROM senses WHERE entry_id = 1000001", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(pos, r#"["noun"]"#);

    let sense_index: i32 = conn
        .query_row("SELECT sense_index FROM senses WHERE entry_id = 1000001", [], |r| r.get(0))
        .unwrap();
    assert_eq!(sense_index, 0);

    let (freq_score, has_common): (i32, i32) = conn
        .query_row("SELECT freq_score, has_common FROM entries WHERE id = 1000001", [], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })
        .unwrap();
    // ichi1 on kanji_reading + ichi1 on reading → has_common bonus 500 + 10 + 10
    assert_eq!(freq_score, 520);
    assert_eq!(has_common, 1);
}

#[test]
fn insert_populates_fts_index() {
    let conn = connection::open_in_memory().unwrap();
    let repo = EntryRepository::new(&conn);

    repo.insert(&sample_entry()).unwrap();

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM glosses_fts WHERE glosses_fts MATCH 'food'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn insert_stores_reading_restrictions() {
    let conn = connection::open_in_memory().unwrap();
    let repo = EntryRepository::new(&conn);

    let restricted = Reading {
        text: String::from("たべもの"),
        priority: vec![],
        no_kanji: false,
        info: vec![],
    };
    let entry = Entry {
        id: 1000002,
        jlpt: None,
        kanji_readings: vec![KanjiReading {
            kanji: String::from("食べ物"),
            restricted_readings: vec![restricted],
            priority: vec![],
            info: vec![],
        }],
        readings: vec![Reading {
            text: String::from("たべもの"),
            priority: vec![],
            no_kanji: false,
            info: vec![],
        }],
        senses: vec![],
    };

    repo.insert(&entry).unwrap();

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM reading_restrictions", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn insert_stores_examples_and_sentences() {
    let conn = connection::open_in_memory().unwrap();
    let repo = EntryRepository::new(&conn);

    let mut sentences = HashMap::new();
    sentences.insert(String::from("jpn"), String::from("食べ物を食べます。"));
    sentences.insert(String::from("eng"), String::from("I eat food."));

    let entry = Entry {
        id: 1000003,
        jlpt: None,
        kanji_readings: vec![],
        readings: vec![],
        senses: vec![Sense {
            pos: vec![],
            misc: vec![],
            refs: vec![],
            glosses: vec![],
            info: vec![],
            dialects: vec![],
            examples: vec![Example {
                source_: Source {
                    name: String::from("tat"),
                    id: Some(String::from("12345")),
                },
                text: String::from("食べ物を食べます。"),
                sentences,
            }],
        }],
    };

    repo.insert(&entry).unwrap();

    let sentence_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM example_sentences", [], |r| r.get(0))
        .unwrap();
    assert_eq!(sentence_count, 2);
}

#[test]
fn insert_stores_cross_references() {
    let conn = connection::open_in_memory().unwrap();
    let repo = EntryRepository::new(&conn);

    let entry = Entry {
        id: 1000004,
        jlpt: None,
        kanji_readings: vec![],
        readings: vec![],
        senses: vec![Sense {
            pos: vec![],
            misc: vec![],
            refs: vec![CrossReference {
                reference: String::from("食べる"),
                reading: Some(String::from("たべる")),
                sense_idx: Some(1),
            }],
            glosses: vec![],
            info: vec![],
            dialects: vec![],
            examples: vec![],
        }],
    };

    repo.insert(&entry).unwrap();

    let (reference, reading, sense_idx): (String, String, i64) = conn
        .query_row(
            "SELECT reference, reading, sense_idx FROM cross_references",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    assert_eq!(reference, "食べる");
    assert_eq!(reading, "たべる");
    assert_eq!(sense_idx, 1);
}

#[test]
fn update_entry_jlpt_with_kanji_key() {
    let conn = connection::open_in_memory().unwrap();
    let repo = EntryRepository::new(&conn);
    repo.insert(&sample_entry()).unwrap();

    repo.update_entry_jlpt("食べ物", "たべもの", 4).unwrap();

    let jlpt: Option<u8> = conn
        .query_row("SELECT jlpt FROM entries WHERE id = 1000001", [], |r| r.get(0))
        .unwrap();
    assert_eq!(jlpt, Some(4));
}

#[test]
fn update_entry_jlpt_keeps_minimum() {
    let conn = connection::open_in_memory().unwrap();
    let repo = EntryRepository::new(&conn);
    repo.insert(&sample_entry()).unwrap();

    repo.update_entry_jlpt("食べ物", "たべもの", 4).unwrap();
    repo.update_entry_jlpt("食べ物", "たべもの", 3).unwrap();
    repo.update_entry_jlpt("食べ物", "たべもの", 5).unwrap();

    let jlpt: Option<u8> = conn
        .query_row("SELECT jlpt FROM entries WHERE id = 1000001", [], |r| r.get(0))
        .unwrap();
    assert_eq!(jlpt, Some(3));
}

#[test]
fn update_entry_jlpt_with_kana_key() {
    let conn = connection::open_in_memory().unwrap();
    let repo = EntryRepository::new(&conn);
    repo.insert(&sample_entry()).unwrap();

    repo.update_entry_jlpt("たべもの", "たべもの", 5).unwrap();

    let jlpt: Option<u8> = conn
        .query_row("SELECT jlpt FROM entries WHERE id = 1000001", [], |r| r.get(0))
        .unwrap();
    assert_eq!(jlpt, Some(5));
}

#[test]
fn update_kanji_jlpt_keeps_minimum() {
    let conn = connection::open_in_memory().unwrap();
    let kanji_repo = KanjiRepository::new(&conn);

    let kanji = Kanji {
        literal: String::from("食"),
        grade: None,
        stroke_count: 9,
        freq: None,
        jlpt: None,
        on_readings: vec![],
        kun_readings: vec![],
        meanings: vec![],
        nanori: vec![],
    };
    kanji_repo.insert(&kanji).unwrap();

    kanji_repo.update_jlpt("食", 4).unwrap();
    kanji_repo.update_jlpt("食", 3).unwrap();
    kanji_repo.update_jlpt("食", 5).unwrap();

    let jlpt: Option<u8> = conn
        .query_row("SELECT jlpt FROM kanji WHERE literal = '食'", [], |r| r.get(0))
        .unwrap();
    assert_eq!(jlpt, Some(3));
}

#[test]
fn sense_index_reflects_position() {
    let conn = connection::open_in_memory().unwrap();
    let repo = EntryRepository::new(&conn);

    let entry = Entry {
        id: 2000001,
        jlpt: None,
        kanji_readings: vec![],
        readings: vec![],
        senses: vec![
            Sense { pos: vec![], misc: vec![], refs: vec![], glosses: vec![], info: vec![], dialects: vec![], examples: vec![] },
            Sense { pos: vec![], misc: vec![], refs: vec![], glosses: vec![], info: vec![], dialects: vec![], examples: vec![] },
            Sense { pos: vec![], misc: vec![], refs: vec![], glosses: vec![], info: vec![], dialects: vec![], examples: vec![] },
        ],
    };
    repo.insert(&entry).unwrap();

    let indices: Vec<i32> = {
        let mut stmt = conn.prepare("SELECT sense_index FROM senses WHERE entry_id = 2000001 ORDER BY id").unwrap();
        stmt.query_map([], |r| r.get(0)).unwrap().map(|r| r.unwrap()).collect()
    };
    assert_eq!(indices, vec![0, 1, 2]);
}

#[test]
fn freq_score_zero_for_no_priority() {
    let conn = connection::open_in_memory().unwrap();
    let repo = EntryRepository::new(&conn);

    let entry = Entry {
        id: 2000002,
        jlpt: None,
        kanji_readings: vec![KanjiReading {
            kanji: String::from("無"),
            restricted_readings: vec![],
            priority: vec![],
            info: vec![],
        }],
        readings: vec![Reading {
            text: String::from("む"),
            priority: vec![],
            no_kanji: false,
            info: vec![],
        }],
        senses: vec![],
    };
    repo.insert(&entry).unwrap();

    let (freq_score, has_common): (i32, i32) = conn
        .query_row("SELECT freq_score, has_common FROM entries WHERE id = 2000002", [], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })
        .unwrap();
    assert_eq!(freq_score, 0);
    assert_eq!(has_common, 0);
}

#[test]
fn sense_counts_populated() {
    let conn = connection::open_in_memory().unwrap();
    let repo = EntryRepository::new(&conn);

    let entry = Entry {
        id: 2000003,
        jlpt: None,
        kanji_readings: vec![],
        readings: vec![],
        senses: vec![
            Sense {
                pos: vec![],
                misc: vec![],
                refs: vec![],
                glosses: vec![
                    Gloss { text: String::from("apple"), type_: None, lang: Some(String::from("eng")) },
                    Gloss { text: String::from("appel"), type_: None, lang: Some(String::from("dut")) },
                ],
                info: vec![],
                dialects: vec![],
                examples: vec![],
            },
            Sense {
                pos: vec![],
                misc: vec![],
                refs: vec![],
                glosses: vec![
                    Gloss { text: String::from("fruit"), type_: None, lang: Some(String::from("eng")) },
                ],
                info: vec![],
                dialects: vec![],
                examples: vec![],
            },
        ],
    };
    repo.insert(&entry).unwrap();

    let eng_count: i32 = conn
        .query_row(
            "SELECT count FROM entry_sense_counts WHERE entry_id = 2000003 AND lang = 'eng'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(eng_count, 2);

    let dut_count: i32 = conn
        .query_row(
            "SELECT count FROM entry_sense_counts WHERE entry_id = 2000003 AND lang = 'dut'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(dut_count, 1);
}

#[test]
fn sense_counts_deduplicate_senses() {
    let conn = connection::open_in_memory().unwrap();
    let repo = EntryRepository::new(&conn);

    let entry = Entry {
        id: 2000004,
        jlpt: None,
        kanji_readings: vec![],
        readings: vec![],
        senses: vec![Sense {
            pos: vec![],
            misc: vec![],
            refs: vec![],
            glosses: vec![
                Gloss { text: String::from("one"), type_: None, lang: Some(String::from("eng")) },
                Gloss { text: String::from("two"), type_: None, lang: Some(String::from("eng")) },
                Gloss { text: String::from("three"), type_: None, lang: Some(String::from("eng")) },
            ],
            info: vec![],
            dialects: vec![],
            examples: vec![],
        }],
    };
    repo.insert(&entry).unwrap();

    let count: i32 = conn
        .query_row(
            "SELECT count FROM entry_sense_counts WHERE entry_id = 2000004 AND lang = 'eng'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);
}
