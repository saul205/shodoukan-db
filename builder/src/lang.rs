/// Maps a JMDict gloss language code to the equivalent Tatoeba download code.
///
/// JMDict uses ISO 639-2/B (bibliographic) codes for three languages where
/// Tatoeba uses ISO 639-3 (terminological) codes. All other codes are identical
/// between the two systems.
///
/// | JMDict | Tatoeba | Language |
/// |--------|---------|----------|
/// | `fre`  | `fra`   | French   |
/// | `ger`  | `deu`   | German   |
/// | `dut`  | `nld`   | Dutch    |
pub fn to_tatoeba(jmdict_lang: &str) -> &str {
    match jmdict_lang {
        "fre" => "fra",
        "ger" => "deu",
        "dut" => "nld",
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_bibliographic_codes_to_terminological() {
        assert_eq!(to_tatoeba("fre"), "fra");
        assert_eq!(to_tatoeba("ger"), "deu");
        assert_eq!(to_tatoeba("dut"), "nld");
    }

    #[test]
    fn passes_through_matching_codes() {
        assert_eq!(to_tatoeba("spa"), "spa");
        assert_eq!(to_tatoeba("eng"), "eng");
        assert_eq!(to_tatoeba("rus"), "rus");
        assert_eq!(to_tatoeba("hun"), "hun");
        assert_eq!(to_tatoeba("slv"), "slv");
    }
}
