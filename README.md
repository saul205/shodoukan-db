# shodoukan

A Japanese-English dictionary database built in Rust. Downloads and parses [JMDict](https://www.edrdg.org/wiki/index.php/JMdict-EDICT_Dictionary_Project) and [KANJIDIC2](https://www.edrdg.org/wiki/index.php/KANJIDIC_Project) from EDRDG, enriches entries with JLPT levels and radical decomposition, stores KanjiVG stroke images, and writes everything into a local SQLite database optimised for fast lookups and full-text search.

## Architecture

The project is a Cargo workspace with two crates:

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
    │   │   ├── jlpt/            # JLPT level data fetcher and DTOs
    │   │   ├── kanjivg/         # KanjiVG SVG stroke images (GitHub API)
    │   │   ├── radkfile/        # Radical decomposition (kradzip.zip)
    │   │   └── tatoeba/         # Tatoeba multilingual translations
    │   ├── traits/datasource.rs # Datasource<T> trait (fetch + parse)
    │   ├── progress.rs          # Progress bar helpers
    │   └── main.rs              # Pipeline entry point
    └── tests/                   # Unit tests for DTO → domain mappers
```

### `core`

Shared library crate with two layers:

- **Domain models** (`core::domain::models`): `Entry`, `KanjiReading`, `Reading`, `Sense`, `Gloss`, `Example`, `CrossReference`, `Source`, `Kanji`, `Meaning`
- **SQLite infrastructure** (`core::infrastructure::sqlite`): schema DDL, `EntryRepository`, `KanjiRepository`

### `builder`

Binary crate that runs the full ingestion pipeline:

1. Downloads and parses **JMDict** (`JMdict.gz`) — ~400k entries with words, readings, senses, and glosses in 8 languages (English, German, French, Russian, Spanish, Hungarian, Slovenian, Dutch). The multilingual file does not include inline example sentences.
2. Downloads and parses **JMDict examples** (`JMdict_e_examp.gz`) — English-only version with ~170k Tanaka Corpus example sentences embedded in senses.
3. Downloads and parses **KANJIDIC2** — ~13k kanji: readings, meanings, grade, stroke count, frequency.
4. Downloads **JLPT vocabulary and kanji lists**.
5. Creates `shodoukan.sqlite`; inserts all data in transactional bulk operations; pre-computes `freq_score`, `has_common`, `sense_index`, per-language sense counts, and `entry_kanji`; populates the `languages` table from JMDict gloss languages.
6. Inserts example sentences and their Japanese/English source sentences into `examples` and `example_sentences`.
7. Enriches entries and kanji with **JLPT levels** from the downloaded lists.
8. Downloads the latest **KanjiVG** release from GitHub — ~6,500 SVG stroke-order images stored in `kanji_svg`.
9. Downloads **kradzip.zip** from the EDRDG FTP server, extracts RADKFILE and RADKFILE2 (EUC-JP decoded), and populates `radicals` and `kanji_radicals` for both JIS X 0208 and JIS X 0212 kanji sets.
10. Downloads **Tatoeba** per-language sentence files for the languages found in `languages`, and links multilingual translations to the example sentences populated in step 6.

See [docs/schema.md](docs/schema.md) for a full description of the database layout.

## Data sources

| Source | File | Coverage |
|--------|------|----------|
| JMDict | `JMdict.gz` — ftp.edrdg.org | ~400k entries, 8 languages, no inline examples |
| JMDict examples | `JMdict_e_examp.gz` — ftp.edrdg.org | ~170k Tanaka Corpus example sentences (English-only) |
| KANJIDIC2 | `kanjidic2.xml.gz` — edrdg.org | ~13k kanji characters |
| JLPT vocab/kanji | github.com/Bluskyo/JLPT_Vocabulary | N1–N5 level annotations |
| KanjiVG | Latest release `-main.zip` — github.com/KanjiVG/kanjivg | ~6,500 SVG stroke-order images |
| RADKFILE + RADKFILE2 | `kradzip.zip` — ftp.edrdg.org | Radical decomposition, JIS X 0208 + JIS X 0212 |
| Tatoeba | Per-language `.tsv.bz2` — downloads.tatoeba.org | Multilingual sentence translations |

JMDict and KANJIDIC2 are maintained by the [Electronic Dictionary Research and Development Group (EDRDG)](https://www.edrdg.org/) and distributed under a [Creative Commons Attribution-ShareAlike 4.0 licence](https://creativecommons.org/licenses/by-sa/4.0/).

## Dependencies

| Crate         | Purpose                                           |
|---------------|---------------------------------------------------|
| `rusqlite`    | SQLite access (bundled, no system dep)            |
| `reqwest`     | Blocking HTTP downloads (with JSON support)       |
| `flate2`      | Gzip decompression                                |
| `bzip2`       | Bzip2 decompression (Tatoeba archives)            |
| `tar`         | Tar archive extraction (Tatoeba links file)       |
| `zip`         | ZIP extraction (KanjiVG release, kradzip.zip)     |
| `encoding_rs` | EUC-JP → UTF-8 decoding (kradzip.zip contents)   |
| `quick-xml`   | Streaming XML parsing + serde support             |
| `serde`       | Deserialization of XML and JSON DTOs              |
| `serde_json`  | JSON array storage in TEXT columns                |
| `regex`       | Extracting `<!ENTITY>` definitions from JMDict    |

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
