use std::collections::HashMap;
use std::io::BufRead;
use std::io::Read;
use std::sync::LazyLock;
use quick_xml::reader::Reader;
use quick_xml::events::Event;
use flate2::read::GzDecoder;
use regex::Regex;

use core::domain::models::entry::Entry;

use crate::traits::datasource::Datasource;
use crate::datasources::jmdict::iterator::JMDictIterator;
use crate::datasources::jmdict::mappers::JMDictMapper;

static ENTITY_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"<!ENTITY\s+(\S+)\s+"([^"]+)">"#).unwrap()
});

pub struct JMDictSource {
    pub ds_url: String,
}

impl JMDictSource {
    fn extract_entities<R: BufRead>(reader: &mut Reader<R>) -> HashMap<String, String> {
        let mut entities = HashMap::new();
        let mut buf = Vec::new();

        loop {
            buf.clear();
            match reader.read_event_into(&mut buf) {
                Ok(Event::DocType(dt)) => {
                    let xml_content = String::from_utf8_lossy(&dt);
                    for cap in ENTITY_REGEX.captures_iter(&xml_content) {
                        entities.insert(cap[1].to_string(), cap[2].to_string());
                    }
                }
                Ok(Event::Start(_)) | Ok(Event::Eof) => break,
                Err(_) => break,
                _ => (),
            }
        }

        entities
    }
}

impl Datasource for JMDictSource {
    type Entity = Entry;

    fn url(&self) -> &str {
        &self.ds_url
    }

    fn fetch(&self) -> Box<dyn std::io::BufRead> {
        println!("Downloading JMDict...");
        let bytes = crate::http::fetch_bytes(self.url());
        let mut decoder = GzDecoder::new(bytes.as_slice());
        let mut content = String::new();
        println!("Decompressing JMDict...");
        decoder.read_to_string(&mut content).unwrap();
        println!("Decompression complete.");
        Box::new(std::io::Cursor::new(content))
    }

    fn parse<R: BufRead>(&self, reader: R) -> Vec<Self::Entity> {
        let mut reader = Reader::from_reader(reader);
        let entities = Self::extract_entities(&mut reader);
        println!("Loaded {} entity definitions", entities.len());

        let iter = JMDictIterator::new(reader, entities);
        let mapper = JMDictMapper::new();
        let entries: Vec<Entry> = iter
            .map(|dto| mapper.map_entry_to_domain(dto))
            .collect();

        entries
    }
}
