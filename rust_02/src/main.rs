use std::env;
use std::fs::{OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};

fn usage() {
    println!("Usage: hextool [OPTIONS]");
    println!("  -f, --file FILE");
    println!("  -r, --read");
    println!("  -w, --write HEXSTRING");
    println!("  -o, --offset N");
    println!("  -s, --size N");
    println!("  -h, --help");
}

fn parse_offset(s: &str) -> u64 {
    if s.starts_with("0x") {
        u64::from_str_radix(&s[2..], 16).unwrap_or(0)
    } else {
        s.parse().unwrap_or(0)
    }
}

fn hex_to_bytes(hex: &str) -> Vec<u8> {
    let mut out = Vec::new();
    let chars = hex.as_bytes().chunks(2);
    for pair in chars {
        if pair.len() == 2 {
            let s = std::str::from_utf8(pair).unwrap();
            if let Ok(v) = u8::from_str_radix(s, 16) {
                out.push(v);
            }
        }
    }
    out
}

fn print_hexdump(bytes: &[u8], start: u64) {
    let mut i = 0usize;
    while i < bytes.len() {
        print!("{:08x}: ", start + i as u64);
        for j in 0..16 {
            if i + j < bytes.len() {
                print!("{:02x} ", bytes[i + j]);
            } else {
                print!("   ");
            }
        }
        print!("|");
        for j in 0..16 {
            if i + j < bytes.len() {
                let c = bytes[i + j];
                if c >= 0x20 && c <= 0x7E {
                    print!("{}", c as char);
                } else {
                    print!(".");
                }
            }
        }
        println!("|");
        i += 16;
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        usage();
        return;
    }

    let mut file = String::new();
    let mut read_mode = false;
    let mut write_hex = String::new();
    let mut offset = 0u64;
    let mut size = 0usize;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => { usage(); return; }
            "-f" | "--file" => { file = args[i+1].clone(); i += 1; }
            "-r" | "--read" => read_mode = true,
            "-w" | "--write" => { write_hex = args[i+1].clone(); i += 1; }
            "-o" | "--offset" => { offset = parse_offset(&args[i+1]); i += 1; }
            "-s" | "--size" => { size = args[i+1].parse().unwrap_or(0); i += 1; }
            _ => {}
        }
        i += 1;
    }

    if read_mode {
        let mut f = OpenOptions::new().read(true).open(&file).unwrap();
        f.seek(SeekFrom::Start(offset)).unwrap();
        let mut buf = vec![0u8; size];
        let n = f.read(&mut buf).unwrap();
        print_hexdump(&buf[..n], offset);
        return;
    }

    if !write_hex.is_empty() {
        let mut f = OpenOptions::new().read(true).write(true).create(true).open(&file).unwrap();
        let bytes = hex_to_bytes(&write_hex);
        f.seek(SeekFrom::Start(offset)).unwrap();
        f.write_all(&bytes).unwrap();
        println!("Writing {} bytes at offset 0x{:08x}", bytes.len(), offset);
        println!("Hex: {}", write_hex);
        let ascii: String = bytes.iter().map(|b| if *b >= 0x20 && *b <= 0x7E { *b as char } else { '.' }).collect();
        println!("ASCII: {}", ascii);
        println!("✓ Successfully written");
        return;
    }

    usage();
}
