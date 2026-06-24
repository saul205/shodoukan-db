use std::collections::HashMap;

use builder::{
    datasources::{
        jmdict::parser::JMDictSource,
        jlpt::source::JlptSource,
        kanjidic::parser::KanjiDicSource,
        kanjivg::source::KanjiVgSource,
        radkfile::source::RadkfileSource,
        tatoeba::source::TatoebaSource,
    },
    progress::{finish_progress, print_progress, step},
    traits::datasource::Datasource,
};
use core::infrastructure::sqlite::{
    connection,
    repository::{
        EntryRepository, KanjiRepository, build_entry_kanji_relations,
        insert_tatoeba_translations,
    },
};

const TOTAL_STEPS: usize = 9;
const PROGRESS_EVERY: usize = 500;

fn main() {
    // ── Step 1: Fetch source data ─────────────────────────────────────────────

    step(1, TOTAL_STEPS, "Fetching JMDict...");
    let jmdict = JMDictSource {
        ds_url: String::from("http://ftp.edrdg.org/pub/Nihongo/JMdict.gz"),
    };
    let entries = jmdict.parse(jmdict.fetch());
    println!("  {} entries parsed", entries.len());

    step(2, TOTAL_STEPS, "Fetching KANJIDIC2...");
    let kanjidic = KanjiDicSource {
        ds_url: String::from("http://www.edrdg.org/kanjidic/kanjidic2.xml.gz"),
    };
    let kanjis = kanjidic.parse(kanjidic.fetch());
    println!("  {} kanji parsed", kanjis.len());

    step(3, TOTAL_STEPS, "Fetching JLPT data...");
    let jlpt = JlptSource;
    let jlpt_vocab = jlpt.fetch_vocab();
    let jlpt_kanji = jlpt.fetch_kanji();
    println!(
        "  {} vocab entries, {} kanji entries",
        jlpt_vocab.len(),
        jlpt_kanji.len()
    );

    // ── Step 4: Open database ─────────────────────────────────────────────────

    step(4, TOTAL_STEPS, "Creating database...");
    let db_path = "shodoukan.sqlite";
    if std::path::Path::new(db_path).exists() {
        std::fs::remove_file(db_path).expect("failed to remove existing database");
    }
    let mut conn = connection::open(db_path).expect("failed to open database");
    println!("  {}", db_path);

    // ── Step 5: Insert kanji and entries ──────────────────────────────────────

    step(5, TOTAL_STEPS, "Inserting kanji and entries...");
    {
        let tx = conn.transaction().expect("failed to begin transaction");
        let repo = KanjiRepository::new(&tx);
        for (i, kanji) in kanjis.iter().enumerate() {
            if i % PROGRESS_EVERY == 0 {
                print_progress("Kanji", i, kanjis.len());
            }
            repo.insert(kanji).expect("failed to insert kanji");
        }
        tx.commit().expect("failed to commit kanji transaction");
        finish_progress("Kanji", kanjis.len());
    }

    {
        let tx = conn.transaction().expect("failed to begin transaction");
        let repo = EntryRepository::new(&tx);
        for (i, entry) in entries.iter().enumerate() {
            if i % PROGRESS_EVERY == 0 {
                print_progress("Entries", i, entries.len());
            }
            repo.insert(entry).expect("failed to insert entry");
        }
        tx.commit().expect("failed to commit entries transaction");
        finish_progress("Entries", entries.len());
    }

    print!("  Building entry-kanji relations...");
    build_entry_kanji_relations(&mut conn).expect("failed to build entry-kanji relations");
    println!(" done");

    // ── Step 6: JLPT enrichment ───────────────────────────────────────────────

    step(6, TOTAL_STEPS, "Enriching with JLPT levels...");
    {
        let tx = conn.transaction().expect("failed to begin transaction");
        let repo = EntryRepository::new(&tx);
        for (i, v) in jlpt_vocab.iter().enumerate() {
            if i % PROGRESS_EVERY == 0 {
                print_progress("Vocab", i, jlpt_vocab.len());
            }
            repo.update_entry_jlpt(&v.key, &v.reading, v.level)
                .expect("failed to update entry jlpt");
        }
        tx.commit().expect("failed to commit entry jlpt transaction");
        finish_progress("Vocab", jlpt_vocab.len());
    }

    {
        let tx = conn.transaction().expect("failed to begin transaction");
        let repo = KanjiRepository::new(&tx);
        for (i, k) in jlpt_kanji.iter().enumerate() {
            if i % PROGRESS_EVERY == 0 {
                print_progress("Kanji levels", i, jlpt_kanji.len());
            }
            repo.update_jlpt(&k.literal, k.level)
                .expect("failed to update kanji jlpt");
        }
        tx.commit().expect("failed to commit kanji jlpt transaction");
        finish_progress("Kanji levels", jlpt_kanji.len());
    }

    // ── Step 7: KanjiVG stroke images ────────────────────────────────────────

    step(7, TOTAL_STEPS, "Fetching KanjiVG stroke images...");
    let svgs = KanjiVgSource.fetch_and_parse();
    {
        let tx = conn.transaction().expect("failed to begin transaction");
        let repo = KanjiRepository::new(&tx);
        for (i, s) in svgs.iter().enumerate() {
            if i % PROGRESS_EVERY == 0 {
                print_progress("SVGs", i, svgs.len());
            }
            repo.insert_svg(&s.literal, &s.svg).expect("failed to insert SVG");
        }
        tx.commit().expect("failed to commit SVG transaction");
        finish_progress("SVGs", svgs.len());
    }

    // ── Step 8: Radical decomposition ────────────────────────────────────────

    step(8, TOTAL_STEPS, "Fetching radical decomposition (RADKFILE)...");
    let (radicals, kanji_radicals) = RadkfileSource.fetch_and_parse();
    {
        let tx = conn.transaction().expect("failed to begin transaction");
        let repo = KanjiRepository::new(&tx);
        for (i, r) in radicals.iter().enumerate() {
            if i % PROGRESS_EVERY == 0 {
                print_progress("Radicals", i, radicals.len());
            }
            repo.insert_radical(&r.literal, r.strokes)
                .expect("failed to insert radical");
        }
        tx.commit().expect("failed to commit radicals transaction");
        finish_progress("Radicals", radicals.len());
    }
    {
        let tx = conn.transaction().expect("failed to begin transaction");
        let repo = KanjiRepository::new(&tx);
        for (i, kr) in kanji_radicals.iter().enumerate() {
            if i % PROGRESS_EVERY == 0 {
                print_progress("Kanji-radical pairs", i, kanji_radicals.len());
            }
            repo.insert_kanji_radical(&kr.kanji_literal, &kr.radical_literal)
                .expect("failed to insert kanji-radical");
        }
        tx.commit().expect("failed to commit kanji-radical transaction");
        finish_progress("Kanji-radical pairs", kanji_radicals.len());
    }

    // ── Step 9: Tatoeba multilingual translations ─────────────────────────────

    step(9, TOTAL_STEPS, "Fetching Tatoeba translations...");
    let langs: Vec<String> = {
        let mut stmt = conn
            .prepare("SELECT code FROM languages")
            .expect("failed to prepare languages query");
        stmt.query_map([], |r| r.get(0))
            .expect("failed to query languages")
            .filter_map(|r| r.ok())
            .collect()
    };
    println!("  Languages in DB: {}", langs.join(", "));

    let known: HashMap<String, i64> = {
        let mut stmt = conn
            .prepare(
                "SELECT source_id, id FROM examples \
                 WHERE source_name = 'tat' AND source_id IS NOT NULL",
            )
            .expect("failed to prepare examples query");
        stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))
            .expect("failed to query examples")
            .filter_map(|r| r.ok())
            .collect()
    };
    println!("  {} JMDict-curated Tatoeba sentences found", known.len());

    let translations = TatoebaSource.fetch_translations(&known, &langs);
    {
        let tx = conn.transaction().expect("failed to begin transaction");
        for (i, t) in translations.iter().enumerate() {
            if i % PROGRESS_EVERY == 0 {
                print_progress("Translations", i, translations.len());
            }
            insert_tatoeba_translations(&tx, std::slice::from_ref(t))
                .expect("failed to insert translation");
        }
        tx.commit().expect("failed to commit translations transaction");
        finish_progress("Translations", translations.len());
    }

    println!("\nDatabase built: {} ({:.1} MB)",
        db_path,
        std::fs::metadata(db_path).map(|m| m.len() as f64 / 1_048_576.0).unwrap_or(0.0)
    );
}
