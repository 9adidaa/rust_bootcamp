use std::env;
fn main() {
    let args: Vec<String> = env::args().collect();
    dbg!(&args);
    let mut count = 0u32;
    println!("{}",&args.len());
    if args.len() == 1 {
        println!("Hello, world!");
    }
    if args.len() == 2 {
        if args[1] == "--help" || args[1] == "-h" {
            println!("Usage: hello [OPTIONS] [NAME]\n\nArguments:\n[NAME] Name to greet [default: World]\n\nOptions:\n--upper Convert to uppercase\n--repeat Repeat greeting N times [default: 1]\n-h, --help Print help");
        } else if args[1] != "--repeat" && args[1] != "--upper"{
            println!("Hello, world! {}", &args[1]);
        }
    }

    if args[1] == "--upper" {
        if args.len() == 3  {
            println!("HELLO, WORLD! {}", &args[2].to_uppercase());
        } else {
            println!("you have to enter a word after --upper");
        }
    }

    if args[1] == "--repeat" {
                if args.len() >= 3 {

                if args[2].chars().all(|c| c.is_numeric()) {
                        if args.len() == 4 {
                        loop {
                            count += 1;
                            println!("Hello, {}!", &args[3]);
                            if args[2] == count.to_string() {
                                break;
                            }
                        }
                    }else{
                        loop {
                            count += 1;
                            println!("Hello, world!");
                            if args[2] == count.to_string() {
                                break;
                            }
                        }
                    }
                } else {
                            println!("Hello, {}!", &args[2]);
                }
            } else {
                println!("you have to enter a number after --repeat");
            }
        }

}
