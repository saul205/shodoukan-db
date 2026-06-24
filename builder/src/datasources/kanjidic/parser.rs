use std::io::BufRead;
use std::io::Read;
use quick_xml::reader::Reader;
use flate2::read::GzDecoder;

use core::domain::models::kanji::Kanji;

use crate::traits::datasource::Datasource;
use crate::datasources::kanjidic::iterator::KanjiDicIterator;
use crate::datasources::kanjidic::mappers::KanjiDicMapper;

pub struct KanjiDicSource {
    pub ds_url: String,
}

impl Datasource for KanjiDicSource {
    type Entity = Kanji;

    fn url(&self) -> &str {
        &self.ds_url
    }

    fn fetch(&self) -> Box<dyn std::io::BufRead> {
        println!("Downloading KANJIDIC2...");
        let bytes = crate::http::fetch_bytes(self.url());
        let mut decoder = GzDecoder::new(bytes.as_slice());
        let mut content = String::new();
        println!("Decompressing KANJIDIC2...");
        decoder.read_to_string(&mut content).unwrap();
        println!("Decompression complete.");
        Box::new(std::io::Cursor::new(content))
    }

    fn parse<R: BufRead>(&self, reader: R) -> Vec<Self::Entity> {
        let reader = Reader::from_reader(reader);
        let iter = KanjiDicIterator::new(reader);
        let mapper = KanjiDicMapper::new();
        let kanjis: Vec<Kanji> = iter
            .map(|dto| mapper.map_character_to_domain(dto))
            .collect();
        println!("Parsed {} kanji", kanjis.len());
        kanjis
    }
}
