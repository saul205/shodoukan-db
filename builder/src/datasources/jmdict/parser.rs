use std::collections::HashMap;
use std::io::BufRead;
use std::io::Read;
use quick_xml::reader::Reader;
use quick_xml::events::Event;
use flate2::read::GzDecoder;
use regex::Regex;

use core::domain::models::entry::Entry;

use crate::traits::datasource::Datasource;
use crate::datasources::jmdict::iterator::JMDictIterator;
use crate::datasources::jmdict::mappers::JMDictMapper;

pub struct JMDictSource {
    pub ds_url: String,
}

impl JMDictSource {

    fn extract_entities<R: BufRead>(reader: &mut Reader<R>) -> HashMap<String, String> {
        let mut entities = HashMap::new();
        let mut buf = Vec::new();

        loop{
            match reader.read_event_into(&mut buf) {
                Ok(Event::DocType(dt)) => {
                    let xml_content = String::from_utf8_lossy(&dt);
                    let re: Regex = Regex::new(r#"<!ENTITY\s+(\S+)\s+"([^"]+)">"#).unwrap();

                    for cap in re.captures_iter(&xml_content) {
                        let name = cap[1].to_string();
                        let value = cap[2].to_string();
                        entities.insert(name, value);
                    }
                },
                Ok(Event::Start(_)) | Ok(Event::Eof) => break,
                Err(_) => break,
                _ => (),
            }
            buf.clear();
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
        println!("Downloading JMDict XML ...");
        let response = reqwest::blocking::get(self.url()).unwrap();
        let mut decoder = GzDecoder::new(response);
        let mut content = String::new();
        println!("Decompressing JMDict XML ...");
        decoder.read_to_string(&mut content).unwrap();
        println!("Decompression complete.");
        Box::new(std::io::Cursor::new(content))
    }

    fn parse<R: BufRead>(&self, reader: R) -> Vec<Self::Entity> {
        let mut reader = Reader::from_reader(reader);
        //reader.trim_text(true);
        let entities = Self::extract_entities(&mut reader);
        
        let iter = JMDictIterator::new(reader);
        let mut entries: Vec<Entry> = Vec::new();
        let mapper = JMDictMapper::new(entities);
        for dto in iter {
            // Map DTO to Domain Entity
            let entry = mapper.map_entry_to_domain(dto);
            println!("Parsed Entry: {:#?}", entry);
            //db.insert_entry(entry);
            entries.push(entry);
        }

        entries
    }
}



