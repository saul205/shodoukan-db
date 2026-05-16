use std::collections::HashMap;
use std::io::BufRead;
use std::io::Cursor;
use quick_xml::de::from_str;
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use quick_xml::writer::Writer;

use crate::datasources::jmdict::dtos::EntryDto;

pub struct JMDictIterator<R: BufRead> {
    reader: Reader<R>,
    buf: Vec<u8>,
    entities: HashMap<String, String>,
}

impl<R: BufRead> JMDictIterator<R> {
    pub fn new(reader: Reader<R>, entities: HashMap<String, String>) -> Self {
        Self { reader, buf: Vec::new(), entities }
    }

    // Replaces custom JMDict entity references (e.g. &n;) with their resolved values.
    // Standard XML entities (&amp; &lt; etc.) are left untouched.
    fn resolve_entities(&self, xml: String) -> String {
        let mut result = xml;
        for (name, value) in &self.entities {
            result = result.replace(&format!("&{};", name), value);
        }
        result
    }
}

impl<R: BufRead> Iterator for JMDictIterator<R> {
    type Item = EntryDto;

    fn next(&mut self) -> Option<Self::Item> {
        let mut entry_writer: Option<Writer<Cursor<Vec<u8>>>> = None;

        loop {
            self.buf.clear();
            match self.reader.read_event_into(&mut self.buf) {
                Ok(Event::Start(ref e)) if e.name().as_ref() == b"entry" => {
                    let mut w = Writer::new(Cursor::new(Vec::new()));
                    w.write_event(Event::Start(e.clone().into_owned())).unwrap();
                    entry_writer = Some(w);
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"entry" => {
                    if let Some(mut w) = entry_writer {
                        w.write_event(Event::End(e.clone().into_owned())).unwrap();
                        let xml = String::from_utf8(w.into_inner().into_inner()).unwrap();
                        let xml = self.resolve_entities(xml);
                        match from_str(&xml) {
                            Ok(entry) => return Some(entry),
                            Err(err) => {
                                let id = xml
                                    .find("<ent_seq>")
                                    .and_then(|s| {
                                        let rest = &xml[s + 9..];
                                        rest.find("</ent_seq>").map(|e| &rest[..e])
                                    })
                                    .unwrap_or("unknown");
                                eprintln!("Skipping entry id={id} — {err}");
                                entry_writer = None;
                                continue;
                            }
                        }
                    }
                    return None;
                }
                Ok(Event::Eof) => return None,
                Ok(ref e) => {
                    if let Some(ref mut w) = entry_writer {
                        w.write_event(e.clone().into_owned()).unwrap();
                    }
                }
                Err(e) => {
                    eprintln!("Error reading XML: {}", e);
                    return None;
                }
            }
        }
    }
}
