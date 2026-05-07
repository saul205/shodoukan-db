use std::io::BufRead;
use std::io::Cursor;
use quick_xml::de::from_str;
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use quick_xml::writer::Writer;

use crate::datasources::kanjidic::dtos::CharacterDto;

pub struct KanjiDicIterator<R: BufRead> {
    reader: Reader<R>,
    buf: Vec<u8>,
}

impl<R: BufRead> KanjiDicIterator<R> {
    pub fn new(reader: Reader<R>) -> Self {
        Self { reader, buf: Vec::new() }
    }
}

impl<R: BufRead> Iterator for KanjiDicIterator<R> {
    type Item = CharacterDto;

    fn next(&mut self) -> Option<Self::Item> {
        let mut entry_writer: Option<Writer<Cursor<Vec<u8>>>> = None;

        loop {
            self.buf.clear();
            match self.reader.read_event_into(&mut self.buf) {
                Ok(Event::Start(ref e)) if e.name().as_ref() == b"character" => {
                    let mut w = Writer::new(Cursor::new(Vec::new()));
                    w.write_event(Event::Start(e.clone().into_owned())).unwrap();
                    entry_writer = Some(w);
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"character" => {
                    if let Some(mut w) = entry_writer {
                        w.write_event(Event::End(e.clone().into_owned())).unwrap();
                        let xml = String::from_utf8(w.into_inner().into_inner()).unwrap();
                        return Some(from_str(&xml).unwrap());
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
                    eprintln!("Error reading KANJIDIC2 XML: {}", e);
                    return None;
                }
            }
        }
    }
}
