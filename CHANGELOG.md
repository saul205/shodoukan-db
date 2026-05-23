# Changelog

All notable changes to this project are documented here.

---

## [Unreleased] — 2026-05-23

### Added
- `idx_glosses_lang_sense` — composite index on `glosses(lang, sense_id)` for efficient language-filtered JOIN queries
- `entry_kanji.priority_score` documented in `docs/schema.md`

### Changed
- Release workflow now uses the git tag as the release ref when the commit is tagged (title always shows the build date)
- `builder` and `core` `Cargo.toml` versions aligned to `0.4.0`

### Planned
- Query interface (API or CLI)
- Search by reading, kanji, or meaning
- Hyperlinks between entries and individual kanji characters

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
