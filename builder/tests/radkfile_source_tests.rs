use builder::datasources::radkfile::source::RadkfileSource;

#[test]
fn radkfile_parses_radical_header() {
    let (radicals, _) = RadkfileSource::parse("# comment\n$ 一 1\n一 万 丁\n");
    assert_eq!(radicals.len(), 1);
    assert_eq!(radicals[0].literal, "一");
    assert_eq!(radicals[0].strokes, 1);
}

#[test]
fn radkfile_parses_kanji_radical_pairs() {
    let (_, pairs) = RadkfileSource::parse("$ 一 1\n一 万 丁\n");
    assert_eq!(pairs.len(), 3);
    assert!(pairs.iter().all(|p| p.radical_literal == "一"));
    let kanjis: Vec<&str> = pairs.iter().map(|p| p.kanji_literal.as_str()).collect();
    assert!(kanjis.contains(&"一"));
    assert!(kanjis.contains(&"万"));
    assert!(kanjis.contains(&"丁"));
}

#[test]
fn radkfile_skips_comment_lines() {
    let (radicals, pairs) = RadkfileSource::parse("# this is a comment\n$ 亅 2\n丁 才\n");
    assert_eq!(radicals.len(), 1);
    assert_eq!(radicals[0].literal, "亅");
    assert_eq!(pairs.len(), 2);
}

#[test]
fn radkfile_handles_multiple_radicals() {
    let (radicals, pairs) = RadkfileSource::parse("$ 一 1\n万 丁\n$ 乙 1\n乙 乾\n");
    assert_eq!(radicals.len(), 2);
    assert_eq!(pairs.len(), 4);
    let rad_lits: Vec<&str> = radicals.iter().map(|r| r.literal.as_str()).collect();
    assert_eq!(rad_lits, ["一", "乙"]);
}
