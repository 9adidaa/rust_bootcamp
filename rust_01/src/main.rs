use std::collections::HashMap;
use std::env;
use std::io::{self, Read};

fn print_help() {
    println!("Usage: wordfreq [OPTIONS]");
    println!("--top N          Show top N words [default: 10]");
    println!("--min-length N   Minimum word length");
    println!("--ignore-case    Case insensitive");
    println!("-h, --help       Show help");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1
        && args[1].starts_with("--")
        && args[1] != "--top"
        && args[1] != "--min-length"
        && args[1] != "--ignore-case"
        && args[1] != "-h"
        && args[1] != "--help"
    {
        eprintln!("error");
        std::process::exit(2);
    }

    if args.len() == 2 && (args[1] == "--help" || args[1] == "-h") {
        print_help();
        return;
    }

    let mut top_n = 10usize;
    let mut min_length = 1usize;
    let mut ignore_case = false;
    let mut text = String::new();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--top" => {
                top_n = args[i + 1].parse().unwrap_or(10);
                i += 1;
            }
            "--min-length" => {
                min_length = args[i + 1].parse().unwrap_or(1);
                i += 1;
            }
            "--ignore-case" => ignore_case = true,
            other => {
                if !other.starts_with('-') {
                    if !text.is_empty() {
                        text.push(' ');
                    }
                    text.push_str(other);
                }
            }
        }
        i += 1;
    }

    if text.is_empty() {
        io::stdin().read_to_string(&mut text).unwrap();
    }

    if ignore_case {
        text = text.to_lowercase();
    }

    let mut freq: HashMap<String, usize> = HashMap::new();

    for w in text.split_whitespace() {
        let clean = w.trim_matches(|c: char| !c.is_alphanumeric());
        if clean.len() >= min_length && !clean.is_empty() {
            *freq.entry(clean.to_string()).or_insert(0) += 1;
        }
    }

    let mut list: Vec<(String, usize)> = freq.into_iter().collect();
    list.sort_by(|a, b| b.1.cmp(&a.1));

    if top_n == 10 {
        println!("Word frequency:");
    } else {
        println!("Top {} words:", top_n);
    }

    for (i, (w, c)) in list.iter().enumerate() {
        if i >= top_n {
            break;
        }
        println!("{}: {}", w, c);
    }
}
