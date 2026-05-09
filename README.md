# shodoukan

A Japanese-English dictionary written in Rust. Downloads the public [JMDict](https://www.edrdg.org/jmdict/j_jmdict.html) and [KANJIDIC2](https://www.edrdg.org/wiki/index.php/KANJIDIC_Project) datasets, processes them, and stores the result in a local SQLite database ready for fast lookups.

## Architecture

The project is a Cargo workspace with two crates:

```
shodoukan/
├── core/       # Domain models and SQLite infrastructure
└── builder/    # Binary: downloads, parses, and ingests the data
```

### `core`

Shared library with two layers:

- **Domain** (`core::domain::models`): plain data structures representing dictionary entries and kanji. No external dependencies.
- **Infrastructure** (`core::infrastructure::sqlite`): database logic — schema, connection, and repositories.

### `builder`

Binary that runs the full data pipeline:

1. Downloads `JMdict.gz` and `kanjidic2.xml.gz` from the EDRDG servers.
2. Decompresses the gzip archives in memory.
3. Parses the XML incrementally (one element at a time) to avoid loading everything into memory.
4. Converts each XML element into a domain model.
5. Inserts all data into SQLite inside transactions.
6. Builds a kanji–entry index with frequency scoring.

### Data flow

```
HTTP (JMDict / KANJIDIC2)
    │
    ▼
Gzip decompression
    │
    ▼
Streaming XML parser
    │
    ▼
DTO → Domain model
    │
    ▼
SQLite (core)
```

## Requirements

- [Rust](https://rustup.rs/) 1.85 or later (2024 edition)
- Internet connection (to download the datasets, ~25 MB total)

SQLite does not need to be installed separately — the project uses a bundled version.

## CI/CD

| Workflow | Trigger | What it does |
|----------|---------|--------------|
| `ci.yml` | Push to `main`, every PR | Runs `cargo test` |
| `release.yml` | 1st of every month (or manual) | Builds `shodoukan.sqlite` and publishes it as a GitHub Release asset |

The database artifact is available under **Releases** with tags like `db-2026-05-09`. You can also trigger a build manually from the Actions tab.

## Installation

```bash
git clone <repository-url>
cd shodoukan
cargo build
```

## Running

```bash
cargo run -p builder
```

The process takes a few minutes. When finished, it produces `shodoukan.sqlite` in the project root.

```
Fetching JMDict...
Downloading JMDict...
Decompressing JMDict...
Loaded 266 entity definitions
Fetching KANJIDIC2...
...
Inserted 13108 kanji
Inserted 401804 entries
Built entry-kanji relations
Database built: shodoukan.sqlite
```

> `shodoukan.sqlite` is not tracked by git. It must be generated locally.

## Tests

```bash
cargo test
```

Includes unit tests for the mappers and integration tests for the repositories against an in-memory database.

## Database

The generated database contains 15 tables and 2 full-text search indexes (FTS5).

See [`docs/schema.md`](docs/schema.md) for a detailed description of each table.

## Key dependencies

| Crate       | Purpose                                        |
|-------------|------------------------------------------------|
| `rusqlite`  | SQLite access (bundled version included)       |
| `reqwest`   | HTTP download of the dataset archives          |
| `flate2`    | Gzip decompression                             |
| `quick-xml` | Streaming XML parser with serde support        |
| `serde`     | XML DTO deserialization                        |
| `regex`     | Extracting entity definitions from JMDict DOCTYPE |

## Data sources

- **JMDict**: maintained by [EDRDG](https://www.edrdg.org/). Licensed under [Creative Commons Attribution-ShareAlike 4.0](https://creativecommons.org/licenses/by-sa/4.0/).
- **KANJIDIC2**: maintained by [EDRDG](https://www.edrdg.org/wiki/index.php/KANJIDIC_Project). Same license.
