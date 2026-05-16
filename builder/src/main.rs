use builder::{
    datasources::{
        jmdict::parser::JMDictSource,
        jlpt::source::JlptSource,
        kanjidic::parser::KanjiDicSource,
    },
    traits::datasource::Datasource,
};
use core::infrastructure::sqlite::{connection, repository::{EntryRepository, KanjiRepository, build_entry_kanji_relations}};

fn main() {
    let jmdict = JMDictSource {
        ds_url: String::from("http://ftp.edrdg.org/pub/Nihongo/JMdict.gz"),
    };
    let kanjidic = KanjiDicSource {
        ds_url: String::from("http://www.edrdg.org/kanjidic/kanjidic2.xml.gz"),
    };
    let jlpt = JlptSource;

    println!("Fetching JMDict...");
    let entries = jmdict.parse(jmdict.fetch());

    println!("Fetching KANJIDIC2...");
    let kanjis = kanjidic.parse(kanjidic.fetch());

    let jlpt_vocab = jlpt.fetch_vocab();
    let jlpt_kanji = jlpt.fetch_kanji();

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

    build_entry_kanji_relations(&conn).expect("failed to build entry-kanji relations");
    println!("Built entry-kanji relations");

    {
        let tx = conn.transaction().expect("failed to begin transaction");
        let repo = EntryRepository::new(&tx);
        for v in &jlpt_vocab {
            repo.update_entry_jlpt(&v.key, &v.reading, v.level)
                .expect("failed to update entry jlpt");
        }
        tx.commit().expect("failed to commit entry jlpt transaction");
    }
    println!("Updated JLPT levels for entries");

    {
        let tx = conn.transaction().expect("failed to begin transaction");
        let repo = KanjiRepository::new(&tx);
        for k in &jlpt_kanji {
            repo.update_jlpt(&k.literal, k.level)
                .expect("failed to update kanji jlpt");
        }
        tx.commit().expect("failed to commit kanji jlpt transaction");
    }
    println!("Updated JLPT levels for kanji");

    println!("Database built: {}", db_path);
}
