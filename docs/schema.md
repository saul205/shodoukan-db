# Database schema

`shodoukan.sqlite` contains 12 regular tables, 2 FTS5 virtual tables, and 2 triggers. Tables are grouped into two domains: dictionary entries (from JMDict) and kanji (from KANJIDIC2). A junction table links both domains.

Arrays (priority tags, part-of-speech, readings, etc.) are stored as JSON strings inside `TEXT` columns (e.g. `'["news1","ichi1"]'`).

---

## Entity-relationship overview

```
entries ──< kanji_readings ──< reading_restrictions >── readings
        ──< readings
        ──< senses ──< glosses ──> glosses_fts (FTS5)
                   ──< cross_references
                   ──< examples ──< example_sentences
        ──< entry_kanji >── kanji ──< kanji_meanings ──> kanji_meanings_fts (FTS5)
```

---

## Dictionary entries (JMDict)

### `entries`

Root table. One row per JMDict entry.

| Column | Type    | Nullable | Description |
|--------|---------|----------|-------------|
| `id`   | INTEGER | NO       | JMDict sequence number (primary key) |
| `jlpt` | INTEGER | YES      | JLPT level 1–5 (1 = N1, 5 = N5). Set from the JLPT enrichment source; `NULL` if not listed |

Indexes: `idx_entries_jlpt` on `jlpt`.

---

### `kanji_readings`

Kanji (written) forms of an entry. An entry may have zero or more.

| Column     | Type    | Nullable | Description |
|------------|---------|----------|-------------|
| `id`       | INTEGER | NO       | Auto-incremented primary key |
| `entry_id` | INTEGER | NO       | FK → `entries(id)` |
| `kanji`    | TEXT    | NO       | Written form (e.g. `食べる`) |
| `priority` | TEXT    | NO       | JSON array of priority codes (e.g. `["news1","ichi1"]`) |
| `info`     | TEXT    | NO       | JSON array of info tags (e.g. `["ateji"]`) |

Indexes: `idx_kanji_readings_entry` on `entry_id`, `idx_kanji_readings_kanji` on `kanji`.

---

### `readings`

Kana (phonetic) readings of an entry. An entry always has at least one.

| Column     | Type    | Nullable | Description |
|------------|---------|----------|-------------|
| `id`       | INTEGER | NO       | Auto-incremented primary key |
| `entry_id` | INTEGER | NO       | FK → `entries(id)` |
| `text`     | TEXT    | NO       | Kana form (e.g. `たべる`) |
| `no_kanji` | INTEGER | NO       | `1` if this reading applies to entries with no kanji form |
| `priority` | TEXT    | NO       | JSON array of priority codes |
| `info`     | TEXT    | NO       | JSON array of info tags |

Indexes: `idx_readings_entry` on `entry_id`, `idx_readings_text` on `text`.

---

### `reading_restrictions`

Maps kana readings to the specific kanji forms they apply to (when a reading does not apply to all kanji forms of an entry).

| Column            | Type    | Nullable | Description |
|-------------------|---------|----------|-------------|
| `reading_id`      | INTEGER | NO       | FK → `readings(id)` |
| `kanji_reading_id`| INTEGER | NO       | FK → `kanji_readings(id)` |

Primary key: `(reading_id, kanji_reading_id)`.

---

### `senses`

One sense = one group of definitions sharing the same part-of-speech, dialect, etc. An entry has one or more senses.

| Column    | Type | Nullable | Description |
|-----------|------|----------|-------------|
| `id`      | INTEGER | NO    | Auto-incremented primary key |
| `entry_id`| INTEGER | NO    | FK → `entries(id)` |
| `pos`     | TEXT    | NO    | JSON array of part-of-speech tags (e.g. `["v1","vt"]`) |
| `misc`    | TEXT    | NO    | JSON array of miscellaneous info tags |
| `dialects`| TEXT    | NO    | JSON array of dialect tags |
| `info`    | TEXT    | NO    | JSON array of sense-level notes |

Index: `idx_senses_entry` on `entry_id`.

---

### `glosses`

English (or other language) translations within a sense.

| Column     | Type    | Nullable | Description |
|------------|---------|----------|-------------|
| `id`       | INTEGER | NO       | Auto-incremented primary key |
| `sense_id` | INTEGER | NO       | FK → `senses(id)` |
| `text`     | TEXT    | NO       | Gloss text (e.g. `"to eat"`) |
| `type`     | TEXT    | YES      | Gloss type (`"expl"`, `"tm"`, etc.) |
| `lang`     | TEXT    | YES      | BCP 47 language tag (default `"eng"`) |

Index: `idx_glosses_sense` on `sense_id`.

Full-text search is available via `glosses_fts` (see below).

---

### `cross_references`

Cross-references from one sense to another entry or reading.

| Column      | Type    | Nullable | Description |
|-------------|---------|----------|-------------|
| `id`        | INTEGER | NO       | Auto-incremented primary key |
| `sense_id`  | INTEGER | NO       | FK → `senses(id)` |
| `reference` | TEXT    | NO       | Target kanji or kana form |
| `reading`   | TEXT    | YES      | Specific reading of the target (if scoped) |
| `sense_idx` | INTEGER | YES      | 1-based sense index within the target entry |

---

### `examples`

Example sentences linked to a sense. Sourced from the Tatoeba corpus embedded in JMDict.

| Column        | Type    | Nullable | Description |
|---------------|---------|----------|-------------|
| `id`          | INTEGER | NO       | Auto-incremented primary key |
| `sense_id`    | INTEGER | NO       | FK → `senses(id)` |
| `source_name` | TEXT    | NO       | Corpus name (e.g. `"tat"`) |
| `source_id`   | TEXT    | YES      | Sentence identifier within the corpus |
| `text`        | TEXT    | NO       | Japanese sentence text |

---

### `example_sentences`

Parallel translations of an example sentence.

| Column       | Type    | Nullable | Description |
|--------------|---------|----------|-------------|
| `id`         | INTEGER | NO       | Auto-incremented primary key |
| `example_id` | INTEGER | NO       | FK → `examples(id)` |
| `lang`       | TEXT    | NO       | BCP 47 language tag (e.g. `"eng"`) |
| `text`       | TEXT    | NO       | Translated sentence text |

---

## Kanji (KANJIDIC2)

### `kanji`

One row per kanji character.

| Column         | Type    | Nullable | Description |
|----------------|---------|----------|-------------|
| `literal`      | TEXT    | NO       | The kanji character (primary key, e.g. `食`) |
| `grade`        | INTEGER | YES      | Joyo grade (1–6 = elementary, 8 = secondary, NULL = non-Joyo) |
| `stroke_count` | INTEGER | NO       | Number of strokes |
| `freq`         | INTEGER | YES      | Frequency rank in newspapers (1 = most frequent) |
| `jlpt`         | INTEGER | YES      | JLPT level 1–5 (set from the JLPT enrichment source) |
| `on_readings`  | TEXT    | NO       | JSON array of on'yomi readings (katakana) |
| `kun_readings` | TEXT    | NO       | JSON array of kun'yomi readings (hiragana) |
| `nanori`       | TEXT    | NO       | JSON array of name readings |

Indexes: `idx_kanji_grade`, `idx_kanji_freq`, `idx_kanji_jlpt`, `idx_kanji_stroke`.

---

### `kanji_meanings`

English (and other language) meanings for a kanji.

| Column    | Type    | Nullable | Description |
|-----------|---------|----------|-------------|
| `id`      | INTEGER | NO       | Auto-incremented primary key |
| `literal` | TEXT    | NO       | FK → `kanji(literal)` |
| `text`    | TEXT    | NO       | Meaning text (e.g. `"eat"`) |
| `lang`    | TEXT    | NO       | BCP 47 language tag (default `"en"`) |

Indexes: `idx_kanji_meanings_literal` on `literal`, `idx_kanji_meanings_lang` on `lang`.

Full-text search is available via `kanji_meanings_fts` (see below).

---

## Junction table

### `entry_kanji`

Links dictionary entries to the individual kanji characters they contain. Populated automatically during entry insertion by scanning `kanji_readings.kanji` for CJK codepoints.

| Column     | Type    | Nullable | Description |
|------------|---------|----------|-------------|
| `entry_id` | INTEGER | NO       | FK → `entries(id)` |
| `literal`  | TEXT    | NO       | A kanji character found in this entry |

Primary key: `(entry_id, literal)`.
Index: `idx_entry_kanji_literal` on `literal` (supports kanji → entries lookups).

---

## Full-text search (FTS5)

### `glosses_fts`

Content-based FTS5 table over `glosses.text`. Populated by trigger on INSERT.

```sql
-- Example usage
SELECT g.text, s.pos, e.id
FROM glosses_fts f
JOIN glosses g ON g.id = f.rowid
JOIN senses s ON s.id = g.sense_id
JOIN entries e ON e.id = s.entry_id
WHERE glosses_fts MATCH 'eat'
ORDER BY rank;
```

### `kanji_meanings_fts`

Content-based FTS5 table over `kanji_meanings.text`. Populated by trigger on INSERT.

```sql
-- Example usage
SELECT k.literal, km.text
FROM kanji_meanings_fts f
JOIN kanji_meanings km ON km.id = f.rowid
JOIN kanji k ON k.literal = km.literal
WHERE kanji_meanings_fts MATCH 'water'
ORDER BY rank;
```

---

## Common query patterns

### Look up an entry by kana reading

```sql
SELECT e.id, e.jlpt, r.text
FROM readings r
JOIN entries e ON e.id = r.entry_id
WHERE r.text = 'たべる';
```

### Get all senses and glosses for an entry

```sql
SELECT s.pos, g.text AS gloss, g.lang
FROM senses s
JOIN glosses g ON g.sense_id = s.id
WHERE s.entry_id = 1169420
ORDER BY s.id, g.id;
```

### Find entries that contain a specific kanji

```sql
SELECT e.id, kr.kanji
FROM entry_kanji ek
JOIN entries e ON e.id = ek.entry_id
JOIN kanji_readings kr ON kr.entry_id = e.id
WHERE ek.literal = '食';
```

### Filter kanji by JLPT level

```sql
SELECT literal, on_readings, kun_readings
FROM kanji
WHERE jlpt = 5
ORDER BY freq;
```
