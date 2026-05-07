use std::collections::HashMap;

use core::domain::models::entry::{
    CrossReference, Entry, Example, Gloss, KanjiReading, Reading, Sense, Source,
};
use core::infrastructure::sqlite::{connection, repository::EntryRepository};

fn sample_entry() -> Entry {
    Entry {
        id: 1000001,
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
