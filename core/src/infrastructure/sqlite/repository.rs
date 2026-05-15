use std::collections::HashMap;
use rusqlite::{Connection, Result, params};
use crate::domain::models::entry::Entry;
use crate::domain::models::kanji::Kanji;

// ── Entry ────────────────────────────────────────────────────────────────────

pub struct EntryRepository<'a> {
    conn: &'a Connection,
}

impl<'a> EntryRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn insert(&self, entry: &Entry) -> Result<()> {
        self.conn.execute(
            "INSERT INTO entries (id, jlpt) VALUES (?1, ?2)",
            params![entry.id, entry.jlpt],
        )?;
        self.insert_readings(entry)?;
        self.insert_senses(entry)?;
        Ok(())
    }

    pub fn update_entry_jlpt(&self, key: &str, reading: &str, level: u8) -> Result<()> {
        if key.chars().any(is_cjk) {
            self.conn.execute(
                "UPDATE entries
                 SET jlpt = CASE WHEN jlpt IS NULL OR ?1 < jlpt THEN ?1 ELSE jlpt END
                 WHERE id IN (
                     SELECT DISTINCT kr.entry_id FROM kanji_readings kr
                     JOIN readings r ON r.entry_id = kr.entry_id
                     WHERE kr.kanji = ?2 AND r.text = ?3
                 )",
                params![level, key, reading],
            )?;
        } else {
            self.conn.execute(
                "UPDATE entries
                 SET jlpt = CASE WHEN jlpt IS NULL OR ?1 < jlpt THEN ?1 ELSE jlpt END
                 WHERE id IN (
                     SELECT DISTINCT entry_id FROM readings WHERE text = ?2
                 )",
                params![level, key],
            )?;
        }
        Ok(())
    }

    fn insert_readings(&self, entry: &Entry) -> Result<()> {
        let mut reading_ids: HashMap<&str, i64> = HashMap::new();

        for r in &entry.readings {
            self.conn.execute(
                "INSERT INTO readings (entry_id, text, no_kanji, priority, info)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![entry.id, r.text, r.no_kanji, json(&r.priority), json(&r.info)],
            )?;
            reading_ids.insert(&r.text, self.conn.last_insert_rowid());
        }

        for kr in &entry.kanji_readings {
            self.conn.execute(
                "INSERT INTO kanji_readings (entry_id, kanji, priority, info)
                 VALUES (?1, ?2, ?3, ?4)",
                params![entry.id, kr.kanji, json(&kr.priority), json(&kr.info)],
            )?;
            let kr_id = self.conn.last_insert_rowid();

            for ch in kr.kanji.chars().filter(|c| is_cjk(*c)) {
                self.conn.execute(
                    "INSERT OR IGNORE INTO entry_kanji (entry_id, literal) VALUES (?1, ?2)",
                    params![entry.id, ch.to_string()],
                )?;
            }

            for r in &kr.restricted_readings {
                if let Some(&r_id) = reading_ids.get(r.text.as_str()) {
                    self.conn.execute(
                        "INSERT INTO reading_restrictions (reading_id, kanji_reading_id)
                         VALUES (?1, ?2)",
                        params![r_id, kr_id],
                    )?;
                }
            }
        }

        Ok(())
    }

    fn insert_senses(&self, entry: &Entry) -> Result<()> {
        for s in &entry.senses {
            self.conn.execute(
                "INSERT INTO senses (entry_id, pos, misc, dialects, info)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![entry.id, json(&s.pos), json(&s.misc), json(&s.dialects), json(&s.info)],
            )?;
            let sense_id = self.conn.last_insert_rowid();

            for g in &s.glosses {
                self.conn.execute(
                    "INSERT INTO glosses (sense_id, text, type, lang) VALUES (?1, ?2, ?3, ?4)",
                    params![sense_id, g.text, g.type_, g.lang],
                )?;
            }

            for cr in &s.refs {
                self.conn.execute(
                    "INSERT INTO cross_references (sense_id, reference, reading, sense_idx)
                     VALUES (?1, ?2, ?3, ?4)",
                    params![sense_id, cr.reference, cr.reading, cr.sense_idx.map(|i| i as i64)],
                )?;
            }

            for ex in &s.examples {
                self.conn.execute(
                    "INSERT INTO examples (sense_id, source_name, source_id, text)
                     VALUES (?1, ?2, ?3, ?4)",
                    params![sense_id, ex.source_.name, ex.source_.id, ex.text],
                )?;
                let example_id = self.conn.last_insert_rowid();

                for (lang, text) in &ex.sentences {
                    self.conn.execute(
                        "INSERT INTO example_sentences (example_id, lang, text)
                         VALUES (?1, ?2, ?3)",
                        params![example_id, lang, text],
                    )?;
                }
            }
        }

        Ok(())
    }
}

// ── Kanji ────────────────────────────────────────────────────────────────────

pub struct KanjiRepository<'a> {
    conn: &'a Connection,
}

impl<'a> KanjiRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn update_jlpt(&self, literal: &str, level: u8) -> Result<()> {
        self.conn.execute(
            "UPDATE kanji
             SET jlpt = CASE WHEN jlpt IS NULL OR ?1 < jlpt THEN ?1 ELSE jlpt END
             WHERE literal = ?2",
            params![level, literal],
        )?;
        Ok(())
    }

    pub fn insert(&self, kanji: &Kanji) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO kanji
             (literal, grade, stroke_count, freq, jlpt, on_readings, kun_readings, nanori)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                kanji.literal,
                kanji.grade,
                kanji.stroke_count,
                kanji.freq,
                kanji.jlpt,
                json(&kanji.on_readings),
                json(&kanji.kun_readings),
                json(&kanji.nanori),
            ],
        )?;

        for m in &kanji.meanings {
            self.conn.execute(
                "INSERT INTO kanji_meanings (literal, text, lang) VALUES (?1, ?2, ?3)",
                params![kanji.literal, m.text, m.lang],
            )?;
        }

        Ok(())
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn json(v: &[String]) -> String {
    serde_json::to_string(v).unwrap_or_else(|_| String::from("[]"))
}

fn is_cjk(c: char) -> bool {
    matches!(c,
        '\u{4E00}'..='\u{9FFF}' |   // CJK Unified Ideographs
        '\u{3400}'..='\u{4DBF}' |   // CJK Extension A
        '\u{F900}'..='\u{FAFF}'     // CJK Compatibility Ideographs
    )
}
