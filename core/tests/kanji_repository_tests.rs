use core::domain::models::kanji::{Kanji, Meaning};
use core::infrastructure::sqlite::{connection, repository::KanjiRepository};

fn sample_kanji() -> Kanji {
    Kanji {
        literal: String::from("本"),
        grade: Some(1),
        stroke_count: 5,
        freq: Some(10),
        jlpt: Some(4),
        on_readings: vec![String::from("ホン")],
        kun_readings: vec![String::from("もと")],
        meanings: vec![
            Meaning { text: String::from("book"), lang: String::from("en") },
            Meaning { text: String::from("main"), lang: String::from("en") },
            Meaning { text: String::from("livre"), lang: String::from("fr") },
        ],
        nanori: vec![String::from("まと")],
    }
}

#[test]
fn insert_populates_kanji_table() {
    let conn = connection::open_in_memory().unwrap();
    let repo = KanjiRepository::new(&conn);

    repo.insert(&sample_kanji()).unwrap();

    let (grade, stroke, freq, jlpt): (i64, i64, i64, i64) = conn
        .query_row(
            "SELECT grade, stroke_count, freq, jlpt FROM kanji WHERE literal = '本'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .unwrap();

    assert_eq!(grade, 1);
    assert_eq!(stroke, 5);
    assert_eq!(freq, 10);
    assert_eq!(jlpt, 4);
}

#[test]
fn insert_stores_readings_as_json() {
    let conn = connection::open_in_memory().unwrap();
    let repo = KanjiRepository::new(&conn);

    repo.insert(&sample_kanji()).unwrap();

    let on: String = conn
        .query_row("SELECT on_readings FROM kanji WHERE literal = '本'", [], |r| r.get(0))
        .unwrap();
    let kun: String = conn
        .query_row("SELECT kun_readings FROM kanji WHERE literal = '本'", [], |r| r.get(0))
        .unwrap();

    assert_eq!(on, r#"["ホン"]"#);
    assert_eq!(kun, r#"["もと"]"#);
}

#[test]
fn insert_populates_meanings_table() {
    let conn = connection::open_in_memory().unwrap();
    let repo = KanjiRepository::new(&conn);

    repo.insert(&sample_kanji()).unwrap();

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM kanji_meanings WHERE literal = '本'", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 3);
}

#[test]
fn insert_populates_meanings_fts() {
    let conn = connection::open_in_memory().unwrap();
    let repo = KanjiRepository::new(&conn);

    repo.insert(&sample_kanji()).unwrap();

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM kanji_meanings_fts WHERE kanji_meanings_fts MATCH 'book'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn duplicate_insert_is_ignored() {
    let conn = connection::open_in_memory().unwrap();
    let repo = KanjiRepository::new(&conn);

    repo.insert(&sample_kanji()).unwrap();
    repo.insert(&sample_kanji()).unwrap();

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM kanji WHERE literal = '本'", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 1);
}
