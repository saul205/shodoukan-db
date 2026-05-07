use quick_xml::reader::Reader;
use quick_xml::events::Event;
use quick_xml::de::from_str;
use quick_xml::writer;
use std::io::BufRead;
use std::io::Cursor;

use crate::datasources::jmdict::dtos::EntryDto;

pub struct JMDictIterator<R: BufRead> {
    reader: Reader<R>,
    buf: Vec<u8>,
}

impl<R: BufRead> JMDictIterator<R> {

    pub fn new(reader: Reader<R>) -> Self {
        Self {
            reader,
            buf: Vec::new(),
        }
    }
}

impl<R: BufRead> Iterator for JMDictIterator<R> {

    type Item = EntryDto; // Assuming EntryDto is defined elsewhere

    fn next(&mut self) -> Option<Self::Item> {
        let mut entry_buf = Vec::new();
        let mut writer = writer::Writer::new(Cursor::new(&mut entry_buf));
        let mut inside_entry = false;
        loop {
            self.buf.clear();
            match self.reader.read_event_into(&mut self.buf) {
                // Find the <entry> tag
                Ok(Event::Start(ref e)) if e.name().as_ref() == b"entry" => {
                    inside_entry = true;
                    writer = writer::Writer::new(Cursor::new(&mut entry_buf));
                    writer.write_event(Event::Start(e.clone().into_owned())).unwrap();
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"entry" => {
                    writer.write_event(Event::End(e.clone().into_owned())).unwrap();
                    let entry_xml = String::from_utf8(entry_buf).unwrap();
                    let clean_xml = entry_xml.replace("&", "&amp;");
                    println!("{}", clean_xml);
                    let entry_dto: EntryDto = from_str(&clean_xml).unwrap();
                    return Some(entry_dto);
                }
                Ok(Event::Eof) => return None,
                Ok(ref e) => {
                    if inside_entry {
                        writer.write_event(e.clone().into_owned()).unwrap();
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