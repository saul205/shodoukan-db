# Changelog

## [Unreleased]

### Planned
- Query interface (API or CLI)
- Search by reading, kanji, or meaning
- Hyperlinks between entries and individual kanji characters

---

## [0.1.0] - 2026-05-09

Initial release. Full database build pipeline.

### Added
- Download and parsing of **JMDict** (~400k entries)
  - Kanji and kana readings, senses, multilingual glosses, cross-references, and examples
  - Custom DOCTYPE entity resolution
- Download and parsing of **KANJIDIC2** (~13k kanji)
  - On/kun readings, meanings, school grade, frequency, JLPT level, and stroke count
- **SQLite** database with 15 tables
  - Full-text search (FTS5) on glosses and kanji meanings
  - `entry_kanji` index with frequency scoring to link entries to individual kanji characters
- Unit tests for mappers and integration tests for repositories
