use std::io::BufRead;

pub trait Datasource {
    /// The Domain Entity this source produces (e.g., Word or Kanji)
    type Entity;

    fn url(&self) -> &str;
    
    /// Returns a stream/reader of the raw data
    fn fetch(&self) -> Box<dyn std::io::BufRead>;

    /// Takes the reader and returns the Iterator
    fn parse<R: BufRead>(&self, reader: R) -> Vec<Self::Entity>;
}