pub const CREATE_SCHEMA: &str = "
    CREATE TABLE IF NOT EXISTS entries (
        id         INTEGER PRIMARY KEY,
        jlpt       INTEGER,
        freq_score INTEGER NOT NULL DEFAULT 0,
        has_common INTEGER NOT NULL DEFAULT 0
    );
    CREATE INDEX IF NOT EXISTS idx_entries_jlpt ON entries(jlpt);

    CREATE TABLE IF NOT EXISTS kanji_readings (
        id       INTEGER PRIMARY KEY AUTOINCREMENT,
        entry_id INTEGER NOT NULL REFERENCES entries(id),
        kanji    TEXT    NOT NULL,
        priority TEXT    NOT NULL DEFAULT '[]',
        info     TEXT    NOT NULL DEFAULT '[]'
    );
    CREATE INDEX IF NOT EXISTS idx_kanji_readings_entry ON kanji_readings(entry_id);
    CREATE INDEX IF NOT EXISTS idx_kanji_readings_kanji ON kanji_readings(kanji);

    CREATE TABLE IF NOT EXISTS readings (
        id       INTEGER PRIMARY KEY AUTOINCREMENT,
        entry_id INTEGER NOT NULL REFERENCES entries(id),
        text     TEXT    NOT NULL,
        no_kanji INTEGER NOT NULL DEFAULT 0,
        priority TEXT    NOT NULL DEFAULT '[]',
        info     TEXT    NOT NULL DEFAULT '[]'
    );
    CREATE INDEX IF NOT EXISTS idx_readings_entry ON readings(entry_id);
    CREATE INDEX IF NOT EXISTS idx_readings_text  ON readings(text);

    CREATE TABLE IF NOT EXISTS reading_restrictions (
        reading_id       INTEGER NOT NULL REFERENCES readings(id),
        kanji_reading_id INTEGER NOT NULL REFERENCES kanji_readings(id),
        PRIMARY KEY (reading_id, kanji_reading_id)
    );

    CREATE TABLE IF NOT EXISTS senses (
        id          INTEGER PRIMARY KEY AUTOINCREMENT,
        entry_id    INTEGER NOT NULL REFERENCES entries(id),
        sense_index INTEGER NOT NULL,
        pos         TEXT    NOT NULL DEFAULT '[]',
        misc        TEXT    NOT NULL DEFAULT '[]',
        dialects    TEXT    NOT NULL DEFAULT '[]',
        info        TEXT    NOT NULL DEFAULT '[]'
    );
    CREATE INDEX IF NOT EXISTS idx_senses_entry ON senses(entry_id);

    CREATE TABLE IF NOT EXISTS glosses (
        id       INTEGER PRIMARY KEY AUTOINCREMENT,
        sense_id INTEGER NOT NULL REFERENCES senses(id),
        text     TEXT    NOT NULL,
        type     TEXT,
        lang     TEXT
    );
    CREATE INDEX IF NOT EXISTS idx_glosses_sense     ON glosses(sense_id);
    CREATE INDEX IF NOT EXISTS idx_glosses_lang_sense ON glosses(lang, sense_id);

    CREATE VIRTUAL TABLE IF NOT EXISTS glosses_fts USING fts5(
        text,
        content='glosses',
        content_rowid='id'
    );

    CREATE TRIGGER IF NOT EXISTS glosses_ai AFTER INSERT ON glosses BEGIN
        INSERT INTO glosses_fts(rowid, text) VALUES (new.id, new.text);
    END;

    CREATE TABLE IF NOT EXISTS cross_references (
        id        INTEGER PRIMARY KEY AUTOINCREMENT,
        sense_id  INTEGER NOT NULL REFERENCES senses(id),
        reference TEXT    NOT NULL,
        reading   TEXT,
        sense_idx INTEGER
    );

    CREATE TABLE IF NOT EXISTS examples (
        id          INTEGER PRIMARY KEY AUTOINCREMENT,
        sense_id    INTEGER NOT NULL REFERENCES senses(id),
        source_name TEXT    NOT NULL,
        source_id   TEXT,
        text        TEXT    NOT NULL
    );

    CREATE TABLE IF NOT EXISTS example_sentences (
        id         INTEGER PRIMARY KEY AUTOINCREMENT,
        example_id INTEGER NOT NULL REFERENCES examples(id),
        lang       TEXT    NOT NULL,
        text       TEXT    NOT NULL
    );

    CREATE TABLE IF NOT EXISTS kanji (
        literal      TEXT    PRIMARY KEY,
        grade        INTEGER,
        stroke_count INTEGER NOT NULL,
        freq         INTEGER,
        jlpt         INTEGER,
        on_readings  TEXT    NOT NULL DEFAULT '[]',
        kun_readings TEXT    NOT NULL DEFAULT '[]',
        nanori       TEXT    NOT NULL DEFAULT '[]'
    );
    CREATE INDEX IF NOT EXISTS idx_kanji_grade  ON kanji(grade);
    CREATE INDEX IF NOT EXISTS idx_kanji_freq   ON kanji(freq);
    CREATE INDEX IF NOT EXISTS idx_kanji_jlpt   ON kanji(jlpt);
    CREATE INDEX IF NOT EXISTS idx_kanji_stroke ON kanji(stroke_count);

    CREATE TABLE IF NOT EXISTS kanji_meanings (
        id      INTEGER PRIMARY KEY AUTOINCREMENT,
        literal TEXT    NOT NULL REFERENCES kanji(literal),
        text    TEXT    NOT NULL,
        lang    TEXT    NOT NULL DEFAULT 'en'
    );
    CREATE INDEX IF NOT EXISTS idx_kanji_meanings_literal ON kanji_meanings(literal);
    CREATE INDEX IF NOT EXISTS idx_kanji_meanings_lang    ON kanji_meanings(lang);

    CREATE VIRTUAL TABLE IF NOT EXISTS kanji_meanings_fts USING fts5(
        text,
        content='kanji_meanings',
        content_rowid='id'
    );

    CREATE TRIGGER IF NOT EXISTS kanji_meanings_ai AFTER INSERT ON kanji_meanings BEGIN
        INSERT INTO kanji_meanings_fts(rowid, text) VALUES (new.id, new.text);
    END;

    CREATE TABLE IF NOT EXISTS entry_kanji (
        entry_id INTEGER NOT NULL REFERENCES entries(id),
        literal  TEXT    NOT NULL,
        PRIMARY KEY (entry_id, literal)
    );
    CREATE INDEX IF NOT EXISTS idx_entry_kanji_literal ON entry_kanji(literal);

    CREATE TABLE IF NOT EXISTS entry_sense_counts (
        entry_id INTEGER NOT NULL REFERENCES entries(id),
        lang     TEXT    NOT NULL,
        count    INTEGER NOT NULL,
        PRIMARY KEY (entry_id, lang)
    );
    CREATE INDEX IF NOT EXISTS idx_entry_sense_counts_entry ON entry_sense_counts(entry_id);
";
