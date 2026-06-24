# Changelog

All notable changes to this project are documented here.

---

## [Unreleased]

### Added

**KanjiVG stroke images**
- `kanji_svg` table — stores full SVG content per kanji character as TEXT; sourced from the latest KanjiVG GitHub release (`-main.zip` asset: non-variant files, all `kvg:` attributes preserved for stroke animation)
- `KanjiVgSource::fetch_and_parse` — queries GitHub API for the latest release URL (no hardcoded version), downloads and extracts the ZIP, returns `Vec<KanjiSvg>`
- `KanjiRepository::insert_svg` — idempotent insert into `kanji_svg`

**Radical decomposition**
- `radicals` table — all radical literals and stroke counts sourced from RADKFILE-u in a single pass
- `kanji_radicals` junction table (M:N) — links `kanji` to `radicals`; both directions indexed (`idx_kanji_radicals_kanji`, `idx_kanji_radicals_radical`)
- `RadkfileSource::fetch_and_parse` — downloads `radkfile-u.gz`, single streaming pass returns `(Vec<Radical>, Vec<KanjiRadical>)`
- `KanjiRepository::insert_radical`, `insert_kanji_radical`

**Multilingual Tatoeba translations**
- `languages` table — auto-populated from every language code encountered during ingestion (JMDict glosses + Tatoeba translations); frontend can query it to discover available filter languages
- `TatoebaSource::fetch_translations` — downloads Tatoeba `links.tar.bz2` to find translations of JMDict-curated sentences, then downloads per-language sentence files for each language in the `languages` table (no hardcoded list)
- `insert_tatoeba_translations` free function
- `insert_language` free function — `INSERT OR IGNORE INTO languages`; called from `EntryRepository::insert_senses` per gloss lang

**Tests**
- `builder/tests/kanjivg_source_tests.rs` — 4 unit tests for `KanjiVgSource::parse_zip` using in-memory ZIPs
- `builder/tests/radkfile_source_tests.rs` — 4 unit tests for `RadkfileSource::parse` using in-memory gzip data

### Changed
- `docs/schema.md` updated: table count 14 → 18, new tables documented, ER diagram updated, new query examples added
- `builder/Cargo.toml`: added `bzip2 = "0.4"`, `tar = "0.4"`, `zip = "2"`, `json` feature on `reqwest`

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
