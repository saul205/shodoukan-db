# Database schema

`shodoukan.sqlite` contains 18 regular tables, 2 FTS5 virtual tables, and 2 triggers. Tables are grouped into two domains: dictionary entries (from JMDict) and kanji (from KANJIDIC2). Junction tables link both domains.

Arrays (priority tags, part-of-speech, readings, etc.) are stored as JSON strings inside `TEXT` columns (e.g. `'["news1","ichi1"]'`).

---

## Entity-relationship overview

```
entries ──< kanji_readings ──< reading_restrictions >── readings
        ──< readings
        ──< senses ──< glosses(lang) ──> glosses_fts (FTS5)
                   ──< cross_references
                   ──< examples ──< example_sentences(lang)   ← currently unpopulated
                   ──< sense_lang_index
        ──< entry_kanji >── kanji ──< kanji_meanings ──> kanji_meanings_fts (FTS5)
        ──< entry_sense_counts                        ──< kanji_svg
                                                      ──< kanji_radicals >── radicals
languages  (populated from lang values in glosses)
```

---

## Dictionary entries (JMDict)

### `entries`

Root table. One row per JMDict entry.

| Column       | Type    | Nullable | Description |
|--------------|---------|----------|-------------|
| `id`         | INTEGER | NO       | JMDict sequence number (primary key) |
| `jlpt`       | INTEGER | YES      | JLPT level 1–5 (1 = N1, 5 = N5). Set from the JLPT enrichment source; `NULL` if not listed |
| `freq_score` | INTEGER | NO       | Pre-computed frequency score. Sum of per-tag scores (Tier 1 tags = 10 pts each, Tier 2 = 5 pts each) across all readings and kanji forms, plus a 500-point bonus if any Tier 1 tag is present (`has_common = 1`). Tier 1: `ichi1`, `spec1`, `news1`, `gai1`. Tier 2: `ichi2`, `spec2`, `news2`, `gai2` |
| `has_common` | INTEGER | NO       | `1` if at least one reading or kanji form carries a Tier 1 priority tag; `0` otherwise |

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

| Column        | Type    | Nullable | Description |
|---------------|---------|----------|-------------|
| `id`          | INTEGER | NO       | Auto-incremented primary key |
| `entry_id`    | INTEGER | NO       | FK → `entries(id)` |
| `sense_index` | INTEGER | NO       | 0-based position of this sense within its entry (first sense = 0) |
| `pos`         | TEXT    | NO       | JSON array of part-of-speech tags (e.g. `["v1","vt"]`) |
| `misc`        | TEXT    | NO       | JSON array of miscellaneous info tags |
| `dialects`    | TEXT    | NO       | JSON array of dialect tags |
| `info`        | TEXT    | NO       | JSON array of sense-level notes |

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

Indexes: `idx_glosses_sense` on `sense_id`, `idx_glosses_lang_sense` on `(lang, sense_id)`.

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

Example sentences linked to a sense. **Currently unpopulated.** `JMdict.gz` (the multilingual distribution) does not embed inline example sentences. The schema is present for future use with `JMdict_e_examp.gz` (English-only JMDict with Tanaka Corpus sentences) or a similar source.

| Column        | Type    | Nullable | Description |
|---------------|---------|----------|-------------|
| `id`          | INTEGER | NO       | Auto-incremented primary key |
| `sense_id`    | INTEGER | NO       | FK → `senses(id)` |
| `source_name` | TEXT    | NO       | Corpus name (e.g. `"tat"` for Tatoeba) |
| `source_id`   | TEXT    | YES      | Sentence identifier within the corpus |
| `text`        | TEXT    | NO       | Japanese sentence text |

---

### `example_sentences`

Parallel translations of an example sentence. **Currently unpopulated** (depends on `examples`; see above).

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

## Junction and auxiliary tables

### `entry_kanji`

Links dictionary entries to the individual kanji characters they contain. Populated automatically during entry insertion by scanning `kanji_readings.kanji` for CJK codepoints.

| Column     | Type    | Nullable | Description |
|------------|---------|----------|-------------|
| `entry_id` | INTEGER | NO       | FK → `entries(id)` |
| `literal`  | TEXT    | NO       | A kanji character found in this entry |

Primary key: `(entry_id, literal)`.
Index: `idx_entry_kanji_literal` on `literal` (supports kanji → entries lookups).

---

### `entry_sense_counts`

Pre-computed count of senses per entry per language. A sense is counted once per language regardless of how many glosses it contains. Populated automatically during entry insertion.

| Column     | Type    | Nullable | Description |
|------------|---------|----------|-------------|
| `entry_id` | INTEGER | NO       | FK → `entries(id)` |
| `lang`     | TEXT    | NO       | BCP 47 language tag (e.g. `"eng"`, `"dut"`) |
| `count`    | INTEGER | NO       | Number of senses that have at least one gloss in this language |

Primary key: `(entry_id, lang)`.
Index: `idx_entry_sense_counts_entry` on `entry_id`.

---

### `sense_lang_index`

Per-language position of a sense within its entry. Whereas `senses.sense_index` is the global 0-based position across all languages, `lang_sense_index` is the 0-based position among the senses of the same entry that have at least one gloss in that specific language. Populated automatically during entry insertion.

| Column            | Type    | Nullable | Description |
|-------------------|---------|----------|-------------|
| `sense_id`        | INTEGER | NO       | FK → `senses(id)` |
| `lang`            | TEXT    | NO       | BCP 47 language tag (e.g. `"eng"`, `"spa"`) |
| `lang_sense_index`| INTEGER | NO       | 0-based position of this sense among senses with at least one gloss in `lang` |

Primary key: `(sense_id, lang)`.
Index: `idx_sense_lang_index_sense` on `sense_id`.

**Example** — 食べる has 10 senses, one of which ("comer") contains only a Spanish gloss and is at global `sense_index = 9`. Its `lang_sense_index` for `spa` is `0` (the first and only Spanish sense), while its `lang_sense_index` for `eng` is not present (no English gloss in that sense).

---

### `languages`

Registry of all language codes present in the database. Populated automatically during JMDict ingestion from the `lang` attribute on `<gloss>` elements — no pre-defined list. After a full build it contains the 8 languages present in `JMdict.gz`: `dut`, `eng`, `fre`, `ger`, `hun`, `rus`, `slv`, `spa`. The Tatoeba step reads this table to decide which per-language sentence files to download. Frontend applications can query this table to discover what filter options to present.

| Column | Type | Nullable | Description |
|--------|------|----------|-------------|
| `code` | TEXT | NO       | ISO 639-3 / BCP 47 language code (primary key, e.g. `"eng"`, `"spa"`) |

---

## Kanji enrichment tables

### `kanji_svg`

SVG stroke-order images sourced from [KanjiVG](https://kanjivg.tagaini.net/) (non-variant files only; all `kvg:` attributes preserved for stroke animation and grid rendering). One row per character. No FK to `kanji` — KanjiVG covers characters outside KANJIDIC2.

| Column    | Type | Nullable | Description |
|-----------|------|----------|-------------|
| `literal` | TEXT | NO       | The character (primary key) |
| `svg`     | TEXT | NO       | Full SVG file content as a UTF-8 string |

---

### `radicals`

All radical literals with their stroke count. Sourced from `kradzip.zip` (EDRDG FTP), which contains RADKFILE (JIS X 0208) and RADKFILE2 (JIS X 0212), decoded from EUC-JP. Covers both radicals that are standalone kanji (e.g. 木, 水) and variant forms that are not (e.g. 亻, 氵). Whether a radical is also a kanji can be determined at query time by joining to `kanji`.

| Column    | Type    | Nullable | Description |
|-----------|---------|----------|-------------|
| `literal` | TEXT    | NO       | The radical character (primary key) |
| `strokes` | INTEGER | NO       | Number of strokes |

---

### `kanji_radicals`

Many-to-many junction linking kanji to the radicals they contain. Populated from RADKFILE. Both lookup directions are indexed: find radicals of a kanji, or find all kanji that use a given radical.

| Column           | Type | Nullable | Description |
|------------------|------|----------|-------------|
| `kanji_literal`  | TEXT | NO       | FK → `kanji(literal)` |
| `radical_literal`| TEXT | NO       | FK → `radicals(literal)` |

Primary key: `(kanji_literal, radical_literal)`.
Indexes: `idx_kanji_radicals_kanji` on `kanji_literal`, `idx_kanji_radicals_radical` on `radical_literal`.

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
SELECT e.id, e.jlpt, e.freq_score, e.has_common, r.text
FROM readings r
JOIN entries e ON e.id = r.entry_id
WHERE r.text = 'たべる';
```

### Get all senses and glosses for an entry (ordered by position)

```sql
SELECT s.sense_index, s.pos, g.text AS gloss, g.lang
FROM senses s
JOIN glosses g ON g.sense_id = s.id
WHERE s.entry_id = 1169420
ORDER BY s.sense_index, g.id;
```

### Get total English sense count for an entry

```sql
SELECT count
FROM entry_sense_counts
WHERE entry_id = 1169420 AND lang = 'eng';
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

### Get radicals of a kanji

```sql
SELECT r.literal, r.strokes
FROM kanji_radicals kr
JOIN radicals r ON r.literal = kr.radical_literal
WHERE kr.kanji_literal = '食'
ORDER BY r.strokes;
```

### Find all kanji that share a radical

```sql
SELECT kr.kanji_literal
FROM kanji_radicals kr
WHERE kr.radical_literal = '食'
ORDER BY kr.kanji_literal;
```

### Check if a radical is also a standalone kanji

```sql
SELECT r.literal, r.strokes, k.literal IS NOT NULL AS is_kanji
FROM radicals r
LEFT JOIN kanji k ON k.literal = r.literal
WHERE r.literal = '人';
```

### List available languages in the database

```sql
SELECT code FROM languages ORDER BY code;
```

### Get Tatoeba translations for an entry's examples

> **Note:** The `examples` table is currently unpopulated (see the `examples` table description above). This query applies once example data is loaded.

```sql
SELECT es.lang, es.text
FROM example_sentences es
JOIN examples e ON e.id = es.example_id
JOIN senses s ON s.id = e.sense_id
WHERE s.entry_id = 1169420
ORDER BY es.lang;
```
