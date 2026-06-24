use std::collections::HashMap;
use rusqlite::{Connection, OptionalExtension, Result, params};
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
        let (freq_score, has_common) = compute_freq_score(entry);
        self.conn.execute(
            "INSERT INTO entries (id, jlpt, freq_score, has_common) VALUES (?1, ?2, ?3, ?4)",
            params![entry.id, entry.jlpt, freq_score, has_common as i32],
        )?;
        self.insert_readings(entry)?;
        self.insert_senses(entry)?;
        self.insert_sense_counts(entry)?;
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
        let lang_idx_map = sense_lang_indices(entry);
        for (index, s) in entry.senses.iter().enumerate() {
            self.conn.execute(
                "INSERT INTO senses (entry_id, sense_index, pos, misc, dialects, info)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![entry.id, index as i32, json(&s.pos), json(&s.misc), json(&s.dialects), json(&s.info)],
            )?;
            let sense_id = self.conn.last_insert_rowid();

            if let Some(lang_entries) = lang_idx_map.get(&index) {
                for (lang, lang_sense_index) in lang_entries {
                    self.conn.execute(
                        "INSERT INTO sense_lang_index (sense_id, lang, lang_sense_index)
                         VALUES (?1, ?2, ?3)",
                        params![sense_id, lang, *lang_sense_index as i32],
                    )?;
                }
            }

            for g in &s.glosses {
                if let Some(lang) = &g.lang {
                    insert_language(self.conn, lang)?;
                }
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

    fn insert_sense_counts(&self, entry: &Entry) -> Result<()> {
        for (lang, count) in count_senses_by_lang(entry) {
            self.conn.execute(
                "INSERT INTO entry_sense_counts (entry_id, lang, count) VALUES (?1, ?2, ?3)",
                params![entry.id, lang, count as i32],
            )?;
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

    pub fn insert_svg(&self, literal: &str, svg: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO kanji_svg (literal, svg) VALUES (?1, ?2)",
            params![literal, svg],
        )?;
        Ok(())
    }

    pub fn insert_radical(&self, literal: &str, strokes: u32) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO radicals (literal, strokes) VALUES (?1, ?2)",
            params![literal, strokes],
        )?;
        Ok(())
    }

    pub fn insert_kanji_radical(&self, kanji_literal: &str, radical_literal: &str) -> Result<()> {
        // Skip pairs where the kanji or radical isn't in our DB (same FK workaround as insert_svg)
        self.conn.execute(
            "INSERT OR IGNORE INTO kanji_radicals (kanji_literal, radical_literal)
             SELECT ?1, ?2
             WHERE EXISTS (SELECT 1 FROM kanji   WHERE literal = ?1)
               AND EXISTS (SELECT 1 FROM radicals WHERE literal = ?2)",
            params![kanji_literal, radical_literal],
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

// ── Language registry ─────────────────────────────────────────────────────────

pub fn insert_language(conn: &Connection, code: &str) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO languages (code) VALUES (?1)",
        params![code],
    )?;
    Ok(())
}

// ── Entry-Kanji Relations ─────────────────────────────────────────────────────

pub fn build_entry_kanji_relations(conn: &mut Connection) -> Result<()> {
    // Collect rows first so the prepared statement is dropped before we open a transaction
    let rows: Vec<(i64, String)> = {
        let mut stmt = conn.prepare("SELECT entry_id, kanji FROM kanji_readings")?;
        stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<Result<_>>()?
    };

    let mut pairs: std::collections::HashSet<(i64, String)> = std::collections::HashSet::new();
    for (entry_id, kanji) in rows {
        for ch in kanji.chars().filter(|c| is_cjk(*c)) {
            pairs.insert((entry_id, ch.to_string()));
        }
    }

    // All inserts in a single transaction — without this each auto-commit hits disk
    let tx = conn.transaction()?;
    for (entry_id, literal) in &pairs {
        tx.execute(
            "INSERT OR REPLACE INTO entry_kanji (entry_id, literal) VALUES (?1, ?2)",
            params![entry_id, literal],
        )?;
    }
    tx.commit()?;

    Ok(())
}

// ── Example sentences ─────────────────────────────────────────────────────────

pub fn insert_entry_examples(conn: &Connection, entry: &Entry) -> Result<()> {
    for (sense_index, sense) in entry.senses.iter().enumerate() {
        if sense.examples.is_empty() { continue; }
        let sense_id: Option<i64> = conn.query_row(
            "SELECT id FROM senses WHERE entry_id = ?1 AND sense_index = ?2",
            params![entry.id as i64, sense_index as i32],
            |r| r.get(0),
        ).optional()?;
        let Some(sense_id) = sense_id else { continue };
        for ex in &sense.examples {
            conn.execute(
                "INSERT INTO examples (sense_id, source_name, source_id, text)
                 VALUES (?1, ?2, ?3, ?4)",
                params![sense_id, ex.source_.name, ex.source_.id, ex.text],
            )?;
            let example_id = conn.last_insert_rowid();
            for (lang, text) in &ex.sentences {
                conn.execute(
                    "INSERT INTO example_sentences (example_id, lang, text)
                     VALUES (?1, ?2, ?3)",
                    params![example_id, lang, text],
                )?;
            }
        }
    }
    Ok(())
}

// ── Tatoeba translations ──────────────────────────────────────────────────────

pub struct TatoebaTranslation {
    pub example_id: i64,
    pub lang: String,
    pub text: String,
}

pub fn insert_tatoeba_translations(conn: &Connection, translations: &[TatoebaTranslation]) -> Result<()> {
    for t in translations {
        conn.execute(
            "INSERT OR IGNORE INTO example_sentences (example_id, lang, text) VALUES (?1, ?2, ?3)",
            params![t.example_id, t.lang, t.text],
        )?;
    }
    Ok(())
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn json(v: &[String]) -> String {
    serde_json::to_string(v).unwrap_or_else(|_| String::from("[]"))
}

fn is_cjk(c: char) -> bool {
    matches!(c,
        '\u{4E00}'..='\u{9FFF}' |
        '\u{3400}'..='\u{4DBF}' |
        '\u{F900}'..='\u{FAFF}'
    )
}

const TIER1: &[&str] = &["ichi1", "spec1", "news1", "gai1"];
const TIER2: &[&str] = &["ichi2", "spec2", "news2", "gai2"];

fn per_tag_score(tags: &[String]) -> i32 {
    let t1 = TIER1.iter().filter(|&&t| tags.iter().any(|p| p == t)).count() as i32;
    let t2 = TIER2.iter().filter(|&&t| tags.iter().any(|p| p == t)).count() as i32;
    t1 * 10 + t2 * 5
}

fn compute_freq_score(entry: &Entry) -> (i32, bool) {
    let has_common = entry.kanji_readings.iter()
        .any(|kr| TIER1.iter().any(|&t| kr.priority.iter().any(|p| p == t)))
        || entry.readings.iter()
        .any(|r| TIER1.iter().any(|&t| r.priority.iter().any(|p| p == t)));
    let bonus = if has_common { 500 } else { 0 };
    let max_kanji = entry.kanji_readings.iter()
        .map(|kr| per_tag_score(&kr.priority))
        .max()
        .unwrap_or(0);
    let max_reading = entry.readings.iter()
        .map(|r| per_tag_score(&r.priority))
        .max()
        .unwrap_or(0);
    (bonus + max_kanji + max_reading, has_common)
}

fn sense_lang_indices(entry: &Entry) -> HashMap<usize, Vec<(String, usize)>> {
    use std::collections::HashSet;
    let mut lang_to_indices: HashMap<String, Vec<usize>> = HashMap::new();
    for (sense_idx, sense) in entry.senses.iter().enumerate() {
        let mut seen: HashSet<&str> = HashSet::new();
        for gloss in &sense.glosses {
            if let Some(lang) = &gloss.lang {
                if seen.insert(lang.as_str()) {
                    lang_to_indices.entry(lang.clone()).or_default().push(sense_idx);
                }
            }
        }
    }
    let mut result: HashMap<usize, Vec<(String, usize)>> = HashMap::new();
    for (lang, indices) in lang_to_indices {
        for (pos, sense_idx) in indices.into_iter().enumerate() {
            result.entry(sense_idx).or_default().push((lang.clone(), pos));
        }
    }
    result
}

fn count_senses_by_lang(entry: &Entry) -> HashMap<String, usize> {
    use std::collections::HashSet;
    let mut lang_senses: HashMap<String, HashSet<usize>> = HashMap::new();
    for (sense_idx, sense) in entry.senses.iter().enumerate() {
        for gloss in &sense.glosses {
            if let Some(lang) = &gloss.lang {
                lang_senses.entry(lang.clone()).or_default().insert(sense_idx);
            }
        }
    }
    lang_senses.into_iter().map(|(lang, ids)| (lang, ids.len())).collect()
}
