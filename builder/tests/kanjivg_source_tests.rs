use std::io::{Cursor, Write};
use zip::write::{SimpleFileOptions, ZipWriter};
use builder::datasources::kanjivg::source::KanjiVgSource;

fn make_zip(entries: &[(&str, &str)]) -> Vec<u8> {
    let buf = Vec::new();
    let cursor = Cursor::new(buf);
    let mut zip = ZipWriter::new(cursor);
    let options = SimpleFileOptions::default();
    for (name, content) in entries {
        zip.start_file(*name, options).unwrap();
        zip.write_all(content.as_bytes()).unwrap();
    }
    zip.finish().unwrap().into_inner()
}

// KanjiVG filenames are zero-padded to 5 hex digits: 食 = U+98DF → 098df.svg

#[test]
fn kanjivg_extracts_literal_from_filename() {
    let bytes = make_zip(&[("098df.svg", "<svg>食</svg>")]);
    let parsed = KanjiVgSource::parse_zip(Cursor::new(bytes));
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0].literal, "食");
    assert_eq!(parsed[0].svg, "<svg>食</svg>");
}

#[test]
fn kanjivg_extracts_multiple_entries() {
    // 水 = U+6C34 → 06c34.svg, 火 = U+706B → 0706b.svg
    let bytes = make_zip(&[
        ("06c34.svg", "<svg>water</svg>"),
        ("0706b.svg", "<svg>fire</svg>"),
    ]);
    let parsed = KanjiVgSource::parse_zip(Cursor::new(bytes));
    assert_eq!(parsed.len(), 2);
    let mut lits: Vec<&str> = parsed.iter().map(|s| s.literal.as_str()).collect();
    lits.sort();
    assert_eq!(lits, ["水", "火"]);
}

#[test]
fn kanjivg_skips_non_svg_files() {
    let bytes = make_zip(&[
        ("098df.svg", "<svg/>"),
        ("README.txt", "ignore me"),
    ]);
    let parsed = KanjiVgSource::parse_zip(Cursor::new(bytes));
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0].literal, "食");
}

#[test]
fn kanjivg_skips_short_stem() {
    // "ab.svg" has stem len 6, below the minimum of 9 (5 hex + ".svg")
    let bytes = make_zip(&[("ab.svg", "<svg/>"), ("098df.svg", "<svg>食</svg>")]);
    let parsed = KanjiVgSource::parse_zip(Cursor::new(bytes));
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0].literal, "食");
}
