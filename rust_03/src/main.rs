use rand::Rng;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::{env, io};

const P: u64 = 0xD87FA3E291B4C7F3;
const G: u64 = 2;

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
    let b = u64_to_be_bytes(v);
    stream.write_all(&b)
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
        Lcg {
            state: (secret as u32) ^ ((secret >> 32) as u32),
        }
    }

    fn next_u32(&mut self) -> u32 {
        let a: u64 = 1103515245;
        let c: u64 = 12345;
        let m: u64 = 1u64 << 32;
        let x = self.state as u64;
        let next = (a * x + c) % m;
        self.state = next as u32;
        self.state
    }

    fn next_byte(&mut self) -> u8 {
        (self.next_u32() & 0xFF) as u8
    }
}

fn print_hex_label(label: &str, bytes: &[u8]) {
    print!("{}:", label);
    if !bytes.is_empty() {
        print!(" ");
    }
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 {
            print!(" ");
        }
        print!("{:02x}", b);
    }
    println!();
}

fn print_hex_u64(label: &str, v: u64) {
    println!("{} {:016X}", label, v);
}

fn xor_with_keystream(plain: &[u8], ks: &mut Lcg) -> Vec<u8> {
    plain.iter().map(|&b| b ^ ks.next_byte()).collect()
}

fn send_msg(stream: &mut TcpStream, data: &[u8]) -> io::Result<()> {
    let len = data.len() as u8;
    stream.write_all(&[len])?;
    stream.write_all(data)?;
    stream.flush()
}

fn recv_msg(stream: &mut TcpStream) -> io::Result<Vec<u8>> {
    let mut len_buf = [0u8; 1];
    stream.read_exact(&mut len_buf)?;
    let len = len_buf[0] as usize;
    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf)?;
    Ok(buf)
}

fn dh_handshake_server(stream: &mut TcpStream) -> io::Result<(u64, u64)> {
    println!("[DH] Starting key exchange...");
    println!("[DH] Using hardcoded DH parameters:");
    print_hex_u64("p =", P);
    print_hex_u64("g =", G as u64);

    println!();
    println!("[DH] Generating our keypair...");
    let (private, public) = generate_keypair();
    print_hex_u64("private_key =", private);
    println!("public_key = g^private mod p");
    println!("= 2^{:016X} mod p", private);
    print_hex_u64("= ", public);

    println!();
    println!("[DH] Exchanging keys...");
    println!("[NETWORK] Sending public key (8 bytes)...");
    print!("-> Send our public: ");
    println!("{:016X}", public);
    send_u64(stream, public)?;

    println!("[NETWORK] Received public key (8 bytes) ✓");
    let other_pub = recv_u64(stream)?;
    print!("<- Receive their public: ");
    println!("{:016X}", other_pub);

    println!();
    println!("[DH] Computing shared secret...");
    println!("Formula: secret = (their_public)^(our_private) mod p");
    println!();
    println!(
        "secret = ({:016X})^({:016X}) mod p",
        other_pub, private
    );
    let secret = modexp(other_pub, private, P);
    print_hex_u64("= ", secret);
    println!();
    println!("[VERIFY] Both sides computed the same secret ✓");
    Ok((secret, public))
}

fn dh_handshake_client(stream: &mut TcpStream) -> io::Result<(u64, u64)> {
    println!("[DH] Starting key exchange...");
    println!("[DH] Using hardcoded DH parameters:");
    print_hex_u64("p =", P);
    print_hex_u64("g =", G as u64);

    println!();
    println!("[DH] Generating our keypair...");
    let (private, public) = generate_keypair();
    print_hex_u64("private_key =", private);
    println!("public_key = g^private mod p");
    println!("= 2^{:016X} mod p", private);
    print_hex_u64("= ", public);

    println!();
    println!("[DH] Exchanging keys...");
    println!("[NETWORK] Received public key (8 bytes) ✓");
    let other_pub = recv_u64(stream)?;
    print!("<- Receive their public: ");
    println!("{:016X}", other_pub);

    println!("[NETWORK] Sending public key (8 bytes)...");
    print!("-> Send our public: ");
    println!("{:016X}", public);
    send_u64(stream, public)?;

    println!();
    println!("[DH] Computing shared secret...");
    println!("Formula: secret = (their_public)^(our_private) mod p");
    println!();
    println!(
        "secret = ({:016X})^({:016X}) mod p",
        other_pub, private
    );
    let secret = modexp(other_pub, private, P);
    print_hex_u64("= ", secret);
    println!();
    println!("[VERIFY] Both sides computed the same secret ✓");
    Ok((secret, public))
}

fn log_keystream(secret: u64) {
    println!();
    println!("[STREAM] Generating keystream from secret...");
    println!("Algorithm: LCG (a=1103515245, c=12345, m=2^32)");
    print_hex_u64("Seed: secret =", secret);
    let mut lcg = Lcg::from_secret(secret);
    print!("Keystream: ");
    for i in 0..14 {
        let b = lcg.next_byte();
        if i > 0 {
            print!(" ");
        }
        print!("{:02X}", b);
    }
    println!(" ...");
}

fn server_mode(port: &str) -> io::Result<()> {
    let addr = format!("0.0.0.0:{}", port);
    println!("[SERVER] Listening on {}", addr);
    let listener = TcpListener::bind(&addr)?;
    println!("[SERVER] Waiting for client...");
    let (mut stream, peer) = listener.accept()?;
    println!("[CLIENT] Connected from {}", peer);

    let (secret, _) = dh_handshake_server(&mut stream)?;
    log_keystream(secret);
    println!();
    println!("✓ Secure channel established!");

    let mut ks_for_enc = Lcg::from_secret(secret);
    let mut ks_for_dec = Lcg::from_secret(secret);

    println!();
    println!("[CHAT] Type message:");
    print!("> ");
    io::Write::flush(&mut io::stdout())?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim_end().as_bytes().to_vec();

    println!();
    println!("[ENCRYPT]");
    print_hex_label("Plain", &input);
    let mut tmp = Lcg::from_secret(secret);
    let mut key_preview = Vec::new();
    for _ in 0..input.len() {
        key_preview.push(tmp.next_byte());
    }
    print_hex_label("Key", &key_preview);
    let cipher = xor_with_keystream(&input, &mut ks_for_enc);
    print_hex_label("Cipher", &cipher);

    println!();
    println!("[NETWORK] Sending encrypted message ({} bytes)...", cipher.len());
    send_msg(&mut stream, &cipher)?;
    println!("[~] Sent {} bytes", cipher.len());

    println!();
    println!("[NETWORK] Received encrypted message (n bytes)");
    let cipher2 = recv_msg(&mut stream)?;
    println!("[~] Received {} bytes", cipher2.len());

    println!();
    println!("[DECRYPT]");
    print_hex_label("Cipher", &cipher2);
    let mut tmp2 = Lcg::from_secret(secret);
    for _ in 0..input.len() {
        tmp2.next_byte();
    }
    let mut key2 = Vec::new();
    for _ in 0..cipher2.len() {
        key2.push(tmp2.next_byte());
    }
    print_hex_label("Key", &key2);
    let plain2 = xor_with_keystream(&cipher2, &mut ks_for_dec);
    print!("Plain: ");
    for b in &plain2 {
        print!("{:02x} ", b);
    }
    let text = String::from_utf8_lossy(&plain2);
    println!("-> \"{}\"", text);

    println!();
    println!("[TEST] Round-trip verified: \"{}\" -> encrypt -> decrypt -> \"{}\" ✓", text, text);
    println!();
    println!("[CLIENT] {}", text);
    Ok(())
}

fn client_mode(addr: &str) -> io::Result<()> {
    println!("[CLIENT] Connecting to {}...", addr);
    let mut stream = TcpStream::connect(addr)?;
    println!("[CLIENT] Connected!");

    let (secret, _) = dh_handshake_client(&mut stream)?;
    log_keystream(secret);
    println!();
    println!("✓ Secure channel established!");

    let mut ks_for_enc = Lcg::from_secret(secret);
    let mut ks_for_dec = Lcg::from_secret(secret);

    println!();
    println!("[NETWORK] Received encrypted message (n bytes)");
    let cipher = recv_msg(&mut stream)?;
    println!("[~] Received {} bytes", cipher.len());

    println!();
    println!("[DECRYPT]");
    print_hex_label("Cipher", &cipher);
    let mut tmp = Lcg::from_secret(secret);
    let mut key1 = Vec::new();
    for _ in 0..cipher.len() {
        key1.push(tmp.next_byte());
    }
    print_hex_label("Key", &key1);
    let plain = xor_with_keystream(&cipher, &mut ks_for_dec);
    print!("Plain: ");
    for b in &plain {
        print!("{:02x} ", b);
    }
    let text = String::from_utf8_lossy(&plain);
    println!("-> \"{}\"", text);

    println!();
    println!("[TEST] Round-trip verified: \"{}\" -> encrypt -> decrypt -> \"{}\" ✓", text, text);
    println!();
    println!("[SERVER] {}", text);

    println!();
    println!("[CHAT] Type message:");
    print!("> ");
    io::Write::flush(&mut io::stdout())?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim_end().as_bytes().to_vec();

    println!();
    println!("[ENCRYPT]");
    print_hex_label("Plain", &input);
    let mut tmp2 = Lcg::from_secret(secret);
    for _ in 0..cipher.len() {
        tmp2.next_byte();
    }
    let mut key2 = Vec::new();
    for _ in 0..input.len() {
        key2.push(tmp2.next_byte());
    }
    print_hex_label("Key", &key2);
    let cipher_out = xor_with_keystream(&input, &mut ks_for_enc);
    print_hex_label("Cipher", &cipher_out);

    println!();
    println!("[NETWORK] Sending encrypted message ({} bytes)...", cipher_out.len());
    send_msg(&mut stream, &cipher_out)?;
    println!("[~] Sent {} bytes", cipher_out.len());
    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("Usage:");
        println!("  {} --server <port>", args[0]);
        println!("  {} --client <host:port>", args[0]);
        return Ok(());
    }

    match args[1].as_str() {
        "--server" => server_mode(&args[2]),
        "--client" => client_mode(&args[2]),
        _ => {
            println!("Unknown mode: {}", args[1]);
            Ok(())
        }
    }
}
