# shodoukan

A Japanese-English dictionary built in Rust. Parses the [JMDict](https://www.edrdg.org/jmdict/j_jmdict.html) dataset and stores it in a local SQLite database for fast lookups.

## Project structure

Cargo workspace with two crates:

```
shodoukan/
├── core/       # Domain models and SQLite infrastructure (SeaORM)
└── builder/    # Binary: downloads, parses, and ingests JMDict data
```

### `core`

Shared library with:

- **Domain models** (`core::domain::models`): `Entry`, `KanjiReading`, `Reading`, `Sense`, `Gloss`, `Example`, `CrossReference`, `Source`
- **Infrastructure** (`core::infrastructure::sqlite`): SeaORM entities and database connection

### `builder`

Binary that:

1. Downloads `JMdict_e_examp.gz` from the EDRDG FTP server
2. Decompresses the gzip stream in memory
3. Extracts XML entity definitions from the DOCTYPE declaration
4. Streams and parses `<entry>` elements one at a time via `JMDictIterator`
5. Maps each `EntryDto` to a `core::domain::models::entry::Entry`
6. Inserts entries into the SQLite database

## Dependencies

| Crate       | Purpose                                   |
|-------------|-------------------------------------------|
| `reqwest`   | Blocking HTTP download of JMDict archive  |
| `flate2`    | Gzip decompression                        |
| `quick-xml` | Streaming XML parsing + serde support     |
| `serde`     | Deserialization of XML DTOs               |
| `regex`     | Extracting `<!ENTITY>` definitions        |
| `sea-orm`   | ORM for SQLite persistence                |

## Building

```bash
cargo build
```

## Running the builder

```bash
cargo run -p builder
```

This will download (~20 MB), decompress, and parse the full JMDict dictionary (~400k entries).

## Data source

JMDict is maintained by the [Electronic Dictionary Research and Development Group (EDRDG)](https://www.edrdg.org/) and is distributed under a [Creative Commons Attribution-ShareAlike 4.0 licence](https://creativecommons.org/licenses/by-sa/4.0/).
