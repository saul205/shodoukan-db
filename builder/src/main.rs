mod datasources;
mod traits;
use crate::{datasources::jmdict::parser::JMDictSource, traits::datasource::Datasource};

fn main() {
    
    let parser = JMDictSource {
        ds_url: String::from("http://ftp.edrdg.org/pub/Nihongo/JMdict_e_examp.gz"),
    };

    let reader = parser.fetch();
    let entries = parser.parse(reader);
    println!("Total Entries Parsed: {}", entries.len());
}
