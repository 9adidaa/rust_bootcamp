use rand::Rng;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::{env, io};

const P: u64 = 0xD87FA3E291B4C7F3;
const G: u64 = 2;

fn print_usage(prog: &str) {
    println!("Usage:");
    println!("  {prog} server <port>");
    println!("  {prog} client <host:port>");
    println!("  {prog} --server <port>");
    println!("  {prog} --client <host:port>");
}

fn modexp(mut base: u64, mut exp: u64, modu: u64) -> u64 {
    let mut result: u64 = 1;
    base %= modu;

    while exp > 0 {
        if exp & 1 == 1 {
            result = ((result as u128 * base as u128) % modu as u128) as u64;
        }
        base = ((base as u128 * base as u128) % modu as u128) as u64;
        exp >>= 1;
    }

    result
}

fn generate_keypair() -> (u64, u64) {
    let mut rng = rand::thread_rng();
    let private: u64 = rng.r#gen::<u64>();
    let public = modexp(G, private, P);
    (private, public)
}

fn u64_to_be_bytes(v: u64) -> [u8; 8] {
    v.to_be_bytes()
}

fn u64_from_be_bytes(b: [u8; 8]) -> u64 {
    u64::from_be_bytes(b)
}

fn send_u64(stream: &mut TcpStream, v: u64) -> io::Result<()> {
    stream.write_all(&u64_to_be_bytes(v))
}

fn recv_u64(stream: &mut TcpStream) -> io::Result<u64> {
    let mut buf = [0u8; 8];
    stream.read_exact(&mut buf)?;
    Ok(u64_from_be_bytes(buf))
}

struct Lcg {
    state: u32,
}

impl Lcg {
    fn from_secret(secret: u64) -> Self {
        let seed = (secret as u32) ^ ((secret >> 32) as u32);
        Lcg { state: seed }
    }

    fn next_byte(&mut self) -> u8 {
        const A: u64 = 1103515245;
        const C: u64 = 12345;
        const M: u64 = 1u64 << 32;

        let x = self.state as u64;
        let next = (A * x + C) % M;

        self.state = next as u32;
        (self.state & 0xFF) as u8
    }
}

fn print_hex_u64(label: &str, v: u64) {
    println!("{label} {:016X}", v);
}

fn dh_handshake_server(stream: &mut TcpStream) -> io::Result<u64> {
    let (private, public) = generate_keypair();
    print_hex_u64("private_key =", private);
    print_hex_u64("public_key =", public);

    send_u64(stream, public)?;
    let other_pub = recv_u64(stream)?;

    let secret = modexp(other_pub, private, P);
    print_hex_u64("secret =", secret);

    Ok(secret)
}

fn dh_handshake_client(stream: &mut TcpStream) -> io::Result<u64> {
    let (private, public) = generate_keypair();
    print_hex_u64("private_key =", private);
    print_hex_u64("public_key =", public);

    let other_pub = recv_u64(stream)?;
    send_u64(stream, public)?;

    let secret = modexp(other_pub, private, P);
    print_hex_u64("secret =", secret);

    Ok(secret)
}

fn log_keystream(secret: u64) {
    let mut ks = Lcg::from_secret(secret);

    print!("[STREAM] Key: ");
    for i in 0..14 {
        if i > 0 {
            print!(" ");
        }
        print!("{:02X}", ks.next_byte());
    }
    println!();
}

fn server_mode(port: &str) -> io::Result<()> {
    print_hex_u64("p =", P);
    print_hex_u64("g =", G);

    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&addr)?;

    let (mut stream, _) = listener.accept()?;
    let secret = dh_handshake_server(&mut stream)?;
    log_keystream(secret);

    Ok(())
}

fn client_mode(addr: &str) -> io::Result<()> {
    let mut stream = TcpStream::connect(addr)?;

    print_hex_u64("p =", P);
    print_hex_u64("g =", G);

    let secret = dh_handshake_client(&mut stream)?;
    log_keystream(secret);

    println!("Secure channel established");
    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("error");
        std::process::exit(2);
    }

    match args[1].as_str() {
        "-h" | "--help" => {
            print_usage(&args[0]);
            Ok(())
        }
        "server" | "--server" => {
            if args.len() != 3 {
                eprintln!("error");
                std::process::exit(2);
            }
            server_mode(&args[2])
        }
        "client" | "--client" => {
            if args.len() != 3 {
                eprintln!("error");
                std::process::exit(2);
            }
            client_mode(&args[2])
        }
        _ => {
            eprintln!("error");
            std::process::exit(2);
        }
    }
}
