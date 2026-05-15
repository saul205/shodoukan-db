# shodoukan

A Japanese-English dictionary database built in Rust. Downloads and parses [JMDict](https://www.edrdg.org/jmdict/j_jmdict.html) and [KANJIDIC2](https://www.edrdg.org/wiki/index.php/KANJIDIC_Project) from EDRDG, enriches entries with JLPT levels, and stores everything in a local SQLite database optimised for fast lookups and full-text search.

## Project structure

Cargo workspace with two crates:

```
shodoukan-db/
├── core/                        # Library: domain models + SQLite infrastructure
│   ├── src/
│   │   ├── domain/models/
│   │   │   ├── entry.rs         # Entry, KanjiReading, Reading, Sense, Gloss, Example
│   │   │   └── kanji.rs         # Kanji, Meaning
│   │   └── infrastructure/sqlite/
│   │       ├── connection.rs    # DB connection setup
│   │       ├── schema.rs        # DDL: tables, indexes, FTS5 triggers
│   │       └── repository.rs   # EntryRepository, KanjiRepository
│   └── tests/                   # Integration tests (in-memory SQLite)
└── builder/                     # Binary: download → parse → insert pipeline
    ├── src/
    │   ├── datasources/
    │   │   ├── jmdict/          # JMDict parser, DTOs, iterator, mappers
    │   │   ├── kanjidic/        # KANJIDIC2 parser, DTOs, iterator, mappers
    │   │   └── jlpt/            # JLPT level data fetcher and DTOs
    │   ├── traits/datasource.rs # Datasource<T> trait (fetch + parse)
    │   └── main.rs              # Pipeline entry point
    └── tests/                   # Unit tests for DTO → domain mappers
```

### `core`

Shared library crate with two layers:

- **Domain models** (`core::domain::models`): `Entry`, `KanjiReading`, `Reading`, `Sense`, `Gloss`, `Example`, `CrossReference`, `Source`, `Kanji`, `Meaning`
- **SQLite infrastructure** (`core::infrastructure::sqlite`): schema DDL, `EntryRepository`, `KanjiRepository`

### `builder`

Binary crate that runs the full ingestion pipeline:

1. Downloads and parses **JMDict** (~400k entries: words, readings, senses, glosses, examples)
2. Downloads and parses **KANJIDIC2** (~13k kanji: readings, meanings, grade, stroke count, frequency)
3. Downloads **JLPT vocabulary and kanji lists** and uses them to set JLPT levels on entries and kanji
4. Creates `shodoukan.sqlite`, inserts all data in transactional bulk operations
5. Populates `entry_kanji` junction table by extracting CJK codepoints from kanji forms

See [docs/schema.md](docs/schema.md) for a full description of the database layout.

## Data sources

| Source | URL | Coverage |
|--------|-----|----------|
| JMDict | ftp.edrdg.org | ~400k dictionary entries |
| KANJIDIC2 | edrdg.org/kanjidic | ~13k kanji characters |
| JLPT vocab/kanji | github.com/Bluskyo/JLPT_Vocabulary | N1–N5 level annotations |

JMDict and KANJIDIC2 are maintained by the [Electronic Dictionary Research and Development Group (EDRDG)](https://www.edrdg.org/) and distributed under a [Creative Commons Attribution-ShareAlike 4.0 licence](https://creativecommons.org/licenses/by-sa/4.0/).

## Dependencies

| Crate        | Purpose                                  |
|--------------|------------------------------------------|
| `rusqlite`   | SQLite access (bundled, no system dep)   |
| `reqwest`    | Blocking HTTP downloads                  |
| `flate2`     | Gzip decompression                       |
| `quick-xml`  | Streaming XML parsing + serde support    |
| `serde`      | Deserialization of XML and JSON DTOs     |
| `serde_json` | JSON array storage in TEXT columns       |
| `regex`      | Extracting `<!ENTITY>` definitions       |

## Build & run

```bash
# Compile
cargo build --release

# Build shodoukan.sqlite (downloads ~25 MB of source data)
cargo run -p builder

# Run all tests (in-memory SQLite, no downloads)
cargo test
```

The generated `shodoukan.sqlite` is roughly 214 MB and supports full-text search over English glosses and kanji meanings via FTS5.

## CI/CD

Automated workflows run on every push and pull request:

- **CI** (`.github/workflows/ci.yml`): `cargo build` + `cargo test`
- **Release** (`.github/workflows/release.yml`): builds `shodoukan.sqlite` on tagged releases and attaches it as a release asset
