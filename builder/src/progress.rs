use std::io::{self, Write};

const BAR_WIDTH: usize = 36;

pub fn print_progress(label: &str, current: usize, total: usize) {
    let filled = if total == 0 { BAR_WIDTH } else { (current * BAR_WIDTH) / total };
    let empty = BAR_WIDTH - filled;
    print!(
        "\r  {:<28} [{}{}] {}/{}    ",
        label,
        "I".repeat(filled),
        ".".repeat(empty),
        current,
        total
    );
    let _ = io::stdout().flush();
}

pub fn finish_progress(label: &str, total: usize) {
    println!(
        "\r  {:<28} [{}] {}    ",
        label,
        "I".repeat(BAR_WIDTH),
        total
    );
}

pub fn step(n: usize, total: usize, label: &str) {
    println!("\n[{}/{}] {}", n, total, label);
}
