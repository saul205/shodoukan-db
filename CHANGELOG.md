# Changelog

All notable changes to this project are documented here.

---

## [1.0.2] — 2026-10-03

### Fixed

- **Kanji JLPT levels** — `kanji.jlpt` mixed KANJIDIC2's pre-2010 4-level scale with the N1–N5 JLPT list, and `KanjiRepository::update_jlpt` kept the lower of the two, so the old value often won (e.g. 電 stored as 4 instead of N5; no kanji had level 5, ~570 of ~2,230 levels were wrong); the JLPT list now always overwrites, and KANJIDIC2 values are converted to the N1–N5 scale (1→N1, 2→N2, 3→N4, 4→N5) and kept only as a fallback for kanji missing from the list (e.g. 分, 身, 無)
- **Release workflow tags** — scheduled releases reused the version tag on `main`'s HEAD (`v1.0.1`) and failed because that release already existed; the version now comes from `builder/Cargo.toml`, manual runs for an unreleased version create `v<version>`, scheduled runs (and manual runs of an already-released version) create a data-refresh release `v<version>-db.<YYYYMMDD>`, and re-running on the same day replaces that release's asset instead of failing

---

## [1.0.1] — 2026-06-25

### Fixed

- **HTTP reliability** — all datasource downloads now use a shared `reqwest` client (`builder/src/http.rs`) with a 30 s connect timeout, 10 min read timeout, and 3 retries with a 10 s delay between attempts; previously a single slow response from the EDRDG FTP server could hang the build indefinitely
- **Tatoeba language codes** — JMDict uses ISO 639-2/B bibliographic codes (`fre`, `ger`, `dut`) while Tatoeba uses ISO 639-3 terminological codes (`fra`, `deu`, `nld`); the mismatch caused French, German, and Dutch translations to be silently skipped with 0 results; fixed via `builder/src/lang.rs` (`to_tatoeba` mapping function)
- **Tatoeba 2-hop traversal** — the Tatoeba step now follows translation-of-translation links; Tanaka Corpus sentences on Tatoeba are Japanese-English pairs, and most other-language translations (Spanish, French, etc.) are added to the English sentence rather than the Japanese one directly; a second streaming pass over the compressed links file resolves these 2-hop paths with no re-download and minimal memory overhead (~148 MB for the compressed buffer)

---

## [1.0.0] — 2026-06-25

### Added

**Example sentences (Tanaka Corpus)**
- Step 2 of the pipeline now fetches and parses `JMdict_e_examp.gz` — the English-only JMDict distribution which embeds ~32k Tanaka Corpus example sentences directly in `<sense>` elements. `JMdict.gz` (the multilingual file) does not include inline examples.
- `insert_entry_examples` free function in `repository.rs` — given an `Entry` from `JMdict_e_examp.gz`, looks up each sense's `id` by `(entry_id, sense_index)` and inserts into `examples` and `example_sentences`; senses with no examples are skipped silently
- `pipeline_tests.rs`: `jmdict_examples_inserted_into_db` — verifies that examples and their `jpn`/`eng` sentence pairs are inserted correctly when calling `insert_entry_examples`

**KanjiVG stroke images**
- `kanji_svg` table — stores full SVG content per kanji character as TEXT; sourced from the latest KanjiVG GitHub release (`-main.zip` asset: non-variant files, all `kvg:` attributes preserved for stroke animation)
- `KanjiVgSource::fetch_and_parse` — queries the GitHub API for the latest release URL (no hardcoded version), downloads and extracts the ZIP, returns `Vec<KanjiSvg>`
- `KanjiRepository::insert_svg` — idempotent insert into `kanji_svg`

**Radical decomposition**
- `radicals` table — all radical literals and stroke counts from RADKFILE and RADKFILE2
- `kanji_radicals` junction table (M:N) — links `kanji` to `radicals`; both directions indexed (`idx_kanji_radicals_kanji`, `idx_kanji_radicals_radical`)
- `RadkfileSource::fetch_and_parse` — downloads `kradzip.zip` from the EDRDG FTP archive (contains both RADKFILE for JIS X 0208 and RADKFILE2 for JIS X 0212); decodes each file from EUC-JP to UTF-8 using `encoding_rs`; single streaming pass returns `(Vec<Radical>, Vec<KanjiRadical>)`
- `KanjiRepository::insert_radical`, `insert_kanji_radical`

**Multilingual Tatoeba translations**
- `languages` table — auto-populated from gloss language codes encountered during JMDict ingestion; frontend can query it to discover available filter languages; Tatoeba step reads it to decide which per-language sentence files to download (no hardcoded list)
- `TatoebaSource::fetch_translations` — downloads Tatoeba `links.tar.bz2` to find translations of JMDict-curated sentences, then downloads per-language `.tsv.bz2` sentence files for each language in the `languages` table; links translations to existing `examples` rows by `source_id`
- `insert_tatoeba_translations` free function
- `insert_language` free function — `INSERT OR IGNORE INTO languages`; called from `EntryRepository::insert_senses` per gloss lang

**Progress display**
- `builder/src/progress.rs` — `step`, `print_progress`, and `finish_progress` helpers for consistent pipeline output

**Tests**
- `builder/tests/kanjivg_source_tests.rs` — 4 unit tests for KanjiVG ZIP parsing using in-memory ZIPs
- `builder/tests/radkfile_source_tests.rs` — 4 unit tests for `RadkfileSource::parse` using in-memory string data

### Changed
- Pipeline step count: 9 → 11 (`TOTAL_STEPS = 11`); steps renumbered to accommodate the new JMdict_e_examp.gz fetch (step 2) and example insertion (step 7)
- `docs/schema.md`: table count 14 → 18, all new tables documented, ER diagram updated, "currently unpopulated" caveats on `examples` and `example_sentences` removed, new query examples added
- `README.md`: pipeline description updated (10 steps), `JMdict_e_examp.gz` added to data sources table, "not currently used" note removed
- `builder/Cargo.toml`: added `bzip2 = "0.4"`, `tar = "0.4"`, `zip = "2"`, `encoding_rs = "0.8"`, `json` feature on `reqwest`

---

## [0.5.1] — 2026-06-20

### Added
- `idx_glosses_lang_sense` — composite index on `glosses(lang, sense_id)` for efficient language-filtered JOIN queries
- `entries.freq_score` — pre-computed frequency score ingested at build time; mirrors the Tier 1 / Tier 2 priority tag weights from the Python search engine (`ichi1`, `spec1`, `news1`, `gai1` = 10 pts each; `ichi2`, `spec2`, `news2`, `gai2` = 5 pts each; 500-point bonus when `has_common` is set)
- `entries.has_common` — `1` if any reading or kanji form carries a Tier 1 priority tag; `0` otherwise
- `senses.sense_index` — 0-based position of each sense within its entry, recorded at insert time
- `entry_sense_counts` table — pre-computed count of senses per entry per language (one row per `(entry_id, lang)` pair); a sense is counted once regardless of how many glosses it contains
- `idx_entry_sense_counts_entry` index on `entry_sense_counts(entry_id)`
- `sense_lang_index` table — per-language 0-based position of each sense within its entry (`lang_sense_index`); a sense with only a Spanish gloss at global position 9 gets `lang_sense_index = 0` for `spa`, enabling correct language-specific ranking without negative scores
- `idx_sense_lang_index_sense` index on `sense_lang_index(sense_id)`

### Changed
- Release workflow now uses the git tag as the release ref when the commit is tagged (title always shows the build date)
- `builder` and `core` `Cargo.toml` versions bumped to `0.5.1`
- `build_entry_kanji_relations` simplified: no longer computes per-entry priority scores; now collects unique `(entry_id, literal)` pairs only
- `docs/schema.md` updated: table count 12 → 14, new columns and tables documented, query examples updated

### Removed
- `entry_kanji.priority_score` column — was unused; removed along with the `score_priority` helper in `repository.rs`

---

## [0.4.0] — 2026-05-15 — JLPT enrichment + test infrastructure + docs

### Added

**JLPT enrichment**
- `builder/src/datasources/jlpt/` — datasource that downloads vocabulary and kanji JLPT level lists from [Bluskyo/JLPT_Vocabulary](https://github.com/Bluskyo/JLPT_Vocabulary)
- `JlptSource::fetch_vocab` — fetches and parses `JLPT_vocab_ALL.json` (kana key, reading, N-level)
- `JlptSource::fetch_kanji` — fetches and parses `JLPT_kanji_ALL.json` (kanji literal → N-level)
- `EntryRepository::update_entry_jlpt` — updates `entries.jlpt` matching on kanji+reading or reading alone; only overwrites if the new level is lower (higher priority)
- `KanjiRepository::update_jlpt` — same minimum-level logic for `kanji.jlpt`
- `entries.jlpt` and `kanji.jlpt` columns in the schema; `idx_entries_jlpt` index

**Test infrastructure**
- `builder/tests/fixtures/jmdict_sample.xml` — 17 real JMDict entries including full DOCTYPE with entity definitions, entries with `re_restr`, `ke_inf`, `re_nokanji`, multi-gloss and multi-sense
- `builder/tests/fixtures/kanjidic_sample.xml` — 20 real KANJIDIC2 characters including multi-language meanings, nanori, and multiple stroke counts
- `builder/tests/pipeline_tests.rs` — 23 end-to-end tests covering the full XML → in-memory SQLite pipeline:
  - JMDict: entity resolution, reading restrictions, `ke_inf`, FTS indexing, `entry_kanji` CJK extraction, reference entry (明白) with all DB fields verified
  - KANJIDIC2: on/kun filtering, nanori, multi-language meanings, FTS indexing, reference kanji (亜) with all DB fields verified
- `scripts/fetch_kanjidic_sample.sh` — helper script to download and extract a KANJIDIC2 sample into the fixtures directory

**Documentation**
- `docs/schema.md` — full database schema reference: all 12 tables, 2 FTS5 virtual tables, 2 triggers, indexes, JSON array conventions, and common query patterns
- `CHANGELOG.md` — this file
- `README.md` — rewritten to reflect current state: KANJIDIC2 and JLPT as data sources, rusqlite replacing SeaORM, updated project structure and pipeline description

### Changed
- Builder pipeline now runs four transactions: kanji insert, entry insert, entry JLPT update, kanji JLPT update
- `builder/Cargo.toml` — added `rusqlite` as dev-dependency (needed by pipeline tests)
- `core/tests/repository_tests.rs` — extended with JLPT update tests (`update_entry_jlpt` with kanji key, kana key, and minimum-level semantics; `update_kanji_jlpt_keeps_minimum`)

---

## [0.3.0] — 2026-05-09 — entry_kanji relationship + CI/CD

### Added
- `entry_kanji` junction table linking entries to the individual kanji characters they contain
- `idx_entry_kanji_literal` index to support kanji → entries reverse lookups
- CJK extraction logic in `EntryRepository::insert_readings`: scans `kanji_readings.kanji` for CJK Unified Ideographs (U+4E00–U+9FFF), Extension A (U+3400–U+4DBF), and CJK Compatibility Ideographs (U+F900–U+FAFF)
- `.github/workflows/ci.yml` — CI pipeline: `cargo build` + `cargo test` on push/PR
- `.github/workflows/release.yml` — Release pipeline: builds `shodoukan.sqlite` on tag and uploads it as a release asset
- `docs/schema.md` — full database schema reference
- `CHANGELOG.md` — this file

### Changed
- `repository_tests.rs` extended with `entry_kanji` assertions

---

## [0.2.0] — 2026-05-08 — KANJIDIC2 ingestion + rusqlite migration

### Added
- `builder/src/datasources/kanjidic/` — streaming parser, DTOs, iterator, and mapper for KANJIDIC2 XML
  - `KanjiDicSource` implements the `Datasource<Kanji>` trait
  - `KanjiDicIterator` reconstructs `<character>` elements from the XML stream
  - `KanjiDicMapper` converts `CharacterDto` → `Kanji` domain model
- `core/src/domain/models/kanji.rs` — `Kanji` and `Meaning` domain structs
- `core/src/infrastructure/sqlite/schema.rs` — full DDL: 12 tables, 16+ indexes, FTS5 virtual tables, and INSERT triggers
- `core/src/infrastructure/sqlite/repository.rs` — `EntryRepository` and `KanjiRepository` with transactional bulk inserts
- `core/src/infrastructure/sqlite/connection.rs` — `open` and `open_in_memory` helpers
- Integration tests in `core/tests/` covering entry and kanji insert/query round-trips
- Unit tests in `builder/tests/` covering JMDict and KANJIDIC2 DTO → domain mappers

### Changed
- Replaced SeaORM with `rusqlite` (bundled) for direct SQLite control and simpler bundling
- Builder pipeline now inserts kanji before entries (foreign-key order for `entry_kanji`)
- `serde_json` used for JSON-array storage in TEXT columns (priority, info, readings)

### Removed
- SeaORM entities and related generated code

---

## [0.1.0] — 2025-12-24 — Initial commit

### Added
- Cargo workspace with `core` (library) and `builder` (binary) crates
- `builder/src/datasources/jmdict/` — initial JMDict parser
  - `JMDictSource` with gzip download via `reqwest` + `flate2`
  - `JMDictIterator` for streaming `<entry>` XML elements
  - DTOs and mapper from `EntryDto` to `Entry` domain model
- Domain models: `Entry`, `KanjiReading`, `Reading`, `Sense`, `Gloss`, `Example`, `CrossReference`, `Source`
- `Datasource<T>` trait unifying fetch + parse for any data source
