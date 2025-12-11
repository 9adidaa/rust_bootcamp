use std::env;

fn print_usage() {
    println!(
        "Usage: hello [OPTIONS] [NAME]

Arguments:
[NAME] Name to greet [default: World]

Options:
--upper Convert to uppercase
--repeat Repeat greeting N times [default: 1]
-h, --help Print help"
    );
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        println!("Hello, World!");
        return;
    }

    if args[1] == "--help" || args[1] == "-h" {
        print_usage();
        return;
    }

    if args.contains(&"--upper".to_string()) && args.contains(&"--repeat".to_string()) {
        let mut name = "World".to_string();
        let mut count = 1u32;

        for i in 1..args.len() {
            if args[i] == "--repeat" && i + 1 < args.len() {
                count = args[i + 1].parse::<u32>().unwrap_or(1);
            } else if !args[i].starts_with("--") {
                name = args[i].clone();
            }
        }

        name = name.to_uppercase();

        for _ in 0..count {
            println!("{}", name);
        }
        return;
    }

    if args[1].starts_with("--") && args[1] != "--upper" && args[1] != "--repeat" {
        eprintln!("error");
        std::process::exit(2);
    }

    if args[1] == "--upper" {
        if args.len() >= 3 {
            println!("{}", args[2].to_uppercase());
        } else {
            eprintln!("error");
            std::process::exit(2);
        }
        return;
    }

    if args[1] == "--repeat" {
        if args.len() >= 3 {
            let n = args[2].parse::<u32>();
            if n.is_err() {
                eprintln!("error");
                std::process::exit(2);
            }
            let n = n.unwrap();

            let name = if args.len() >= 4 {
                args[3].clone()
            } else {
                "World".to_string()
            };

            for _ in 0..n {
                println!("Hello, {}!", name);
            }
            return;
        } else {
            eprintln!("error");
            std::process::exit(2);
        }
    }

    println!("Hello, {}!", args[1]);
}
