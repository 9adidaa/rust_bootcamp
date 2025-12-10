use std::collections::HashMap;
use std::env;
use std::io::{self, Read};

fn print_help() {
    println!("Usage: wordfreq [OPTIONS]");
    println!("Count word frequency in text");
    println!("Arguments:");
    println!("  Text to analyze (or use stdin)");
    println!("Options:");
    println!("  --top N          Show top N words [default: 10]");
    println!("  --min-length N   Ignore words shorter than N [default: 1]");
    println!("  --ignore-case    Case insensitive counting");
    println!("  -h, --help       Show this help message");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 2 && (args[1] == "-h" || args[1] == "--help") {
        print_help();
        return;
    }

    let mut top_n = 10usize;
    let mut min_length = 1usize;
    let mut ignore_case = false;
    let mut text_input = String::new();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--top" => {
                if i + 1 < args.len() {
                    top_n = args[i + 1].parse().unwrap_or(10);
                    i += 1;
                }
            }
            "--min-length" => {
                if i + 1 < args.len() {
                    min_length = args[i + 1].parse().unwrap_or(1);
                    i += 1;
                }
            }
            "--ignore-case" => {
                ignore_case = true;
            }
            other => {
                if !other.starts_with('-') {
                    if !text_input.is_empty() {
                        text_input.push(' ');
                    }
                    text_input.push_str(other);
                }
            }
        }
        i += 1;
    }

    let mut input = String::new();
    if text_input.is_empty() {
        io::stdin().read_to_string(&mut input).unwrap();
    } else {
        input = text_input;
    }

    if ignore_case {
        input = input.to_lowercase();
    }

    let mut map: HashMap<String, usize> = HashMap::new();

    for raw in input.split_whitespace() {
        let w = raw.trim_matches(|c: char| !c.is_alphanumeric());
        if w.len() < min_length || w.is_empty() {
            continue;
        }
        *map.entry(w.to_string()).or_insert(0) += 1;
    }

    let mut vec: Vec<(String, usize)> = map.into_iter().collect();
    vec.sort_by(|a, b| b.1.cmp(&a.1));

    if top_n == 10 {
        println!("Word frequency:");
    } else {
        println!("Top {} words:", top_n);
    }

    for (idx, (word, count)) in vec.iter().enumerate() {
        if idx >= top_n {
            break;
        }
        println!("{}: {}", word, count);
    }
}
