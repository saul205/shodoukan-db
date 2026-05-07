use builder::{datasources::{jmdict::parser::JMDictSource, kanjidic::parser::KanjiDicSource}, traits::datasource::Datasource};
use core::infrastructure::sqlite::{connection, repository::{EntryRepository, KanjiRepository}};

fn main() {
    let jmdict = JMDictSource {
        ds_url: String::from("http://ftp.edrdg.org/pub/Nihongo/JMdict.gz"),
    };
    let kanjidic = KanjiDicSource {
        ds_url: String::from("http://www.edrdg.org/kanjidic/kanjidic2.xml.gz"),
    };

    println!("Fetching JMDict...");
    let entries = jmdict.parse(jmdict.fetch());

    println!("Fetching KANJIDIC2...");
    let kanjis = kanjidic.parse(kanjidic.fetch());

    let db_path = "shodoukan.sqlite";
    if std::path::Path::new(db_path).exists() {
        std::fs::remove_file(db_path).expect("failed to remove existing database");
    }
    let mut conn = connection::open(db_path).expect("failed to open database");

    {
        let tx = conn.transaction().expect("failed to begin transaction");
        let repo = KanjiRepository::new(&tx);
        for kanji in &kanjis {
            repo.insert(kanji).expect("failed to insert kanji");
        }
        tx.commit().expect("failed to commit kanji transaction");
    }
    println!("Inserted {} kanji", kanjis.len());

    {
        let tx = conn.transaction().expect("failed to begin transaction");
        let repo = EntryRepository::new(&tx);
        for entry in &entries {
            repo.insert(entry).expect("failed to insert entry");
        }
        tx.commit().expect("failed to commit entries transaction");
    }
    println!("Inserted {} entries", entries.len());

    println!("Database built: {}", db_path);
}
