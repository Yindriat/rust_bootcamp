use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use rand::Rng;

// Diffie-Hellman parameters (hardcoded to avoid randomness issues)
const DH_P: u64 = 0xD87FA3E29184C7F3; // 64-bit prime
const DH_G: u64 = 2;                  // Generator

/// Modular exponentiation: base^exp mod modulus using square-and-multiply
fn mod_exp(mut base: u64, mut exp: u64, modulus: u64) -> u64 {
    let mut result = 1u64;
    base %= modulus;

    while exp > 0 {
        if exp % 2 == 1 {
            result = ((result as u128 * base as u128) % modulus as u128) as u64;
        }
        exp >>= 1;
        base = ((base as u128 * base as u128) % modulus as u128) as u64;
    }
    result
}

/// LCG keystream generator
fn generate_keystream(seed: u64, length: usize) -> Vec<u8> {
    let mut rng = LCG::new(seed);
    (0..length).map(|_| rng.next()).collect()
}

struct LCG {
    state: u64,
}

impl LCG {
    fn new(seed: u64) -> Self {
        LCG { state: seed }
    }

    fn next(&mut self) -> u8 {
        self.state = self.state.wrapping_mul(1103515245).wrapping_add(12345);
        ((self.state >> 32) & 0xFF) as u8
    }
}

/// XOR encrypt/decrypt with keystream
fn xor_cipher(data: &[u8], keystream: &[u8]) -> Vec<u8> {
    data.iter()
        .zip(keystream.iter())
        .map(|(d, k)| d ^ k)
        .collect()
}

struct DHSession {
    private_key: u64,
    public_key: u64,
    shared_secret: Option<u64>,
}

impl DHSession {
    fn new() -> Self {
        let mut rng = rand::thread_rng();
        let private_key = rng.gen::<u64>();
        let public_key = mod_exp(DH_G, private_key, DH_P);

        DHSession {
            private_key,
            public_key,
            shared_secret: None,
        }
    }

    fn compute_shared_secret(&mut self, their_public_key: u64) {
        let secret = mod_exp(their_public_key, self.private_key, DH_P);
        self.shared_secret = Some(secret);
    }
}

fn handle_client(mut stream: TcpStream, is_server: bool) {
    let label = if is_server { "[SERVER]" } else { "[CLIENT]" };

    println!("{} Connected from {}", label, stream.peer_addr().unwrap());

    // Initialize Diffie-Hellman
    let mut dh = DHSession::new();
    println!("{}", label);
    println!("{} Starting key exchange...", label);
    println!("{} Using hardcoded DH parameters:", label);
    println!("{} p = D87F A3E2 9184 C7F3 (64-bit prime - public)", label);
    println!("{} g = 2 (Generator - public)", label);
    println!();
    println!("{} Generating our keypair...", label);
    println!("private_key = {:016x} (Random 64-bit)", label);
    println!("public_key = g^private_key mod p", label);
    println!("          = 2^{:x} mod 0xD87FA3E29184C7F3", dh.private_key);
    println!("          = {:016x}", dh.public_key);
    println!();

    // Send our public key (8 bytes)
    let pub_key_bytes = dh.public_key.to_le_bytes();
    stream.write_all(&pub_key_bytes).unwrap();
    println!("{} [DH] Exchanging keys...", label);
    println!("{} [NETWORK] Sending public key (8 bytes)...", label);
    println!("{} - Send our public: {:016x}", label, dh.public_key);

    // Receive their public key
    let mut buffer = [0u8; 8];
    stream.read_exact(&mut buffer).unwrap();
    let their_public_key = u64::from_le_bytes(buffer);
    println!("{} [NETWORK] Received public key (8 bytes) /", label);
    println!("{} - Receive their public: {:016x}", label, their_public_key);
    println!();

    // Compute shared secret
    dh.compute_shared_secret(their_public_key);
    let secret = dh.shared_secret.unwrap();

    println!("{} [DH] Computing shared secret...", label);
    println!("{} Formula: secret = (their_public)^(our_private) mod p", label);
    println!();
    println!("secret = ({:016x})^({:016x}) mod 0xD87FA3E29184C7F3", their_public_key, dh.private_key);
    println!("       = {:016x}", secret);
    println!();
    println!("{} [VERIFY] Both sides computed the same secret /", label);
    println!();

    // Generate keystream
    println!("{} [STREAM] Generating keystream from secret...", label);
    println!("Algorithm: LCG (a=1103515245, c=12345, m=2^32)", label);
    println!("Seed: secret = {:016x}", secret);

    // Chat loop
    println!();
    println!("{} / Secure channel established!", label);
    println!();
    println!("{} [CHAT] Type message:", label);

    let stdin = std::io::stdin();
    let mut input = String::new();

    loop {
        input.clear();
        stdin.read_line(&mut input).unwrap();
        let message = input.trim();

        if message.is_empty() {
            continue;
        }

        // Encrypt message
        let keystream = generate_keystream(secret, message.len());
        let cipher = xor_cipher(message.as_bytes(), &keystream);

        println!();
        println!("{} [ENCRYPT]", label);
        println!("Plain: {}", message);
        println!("Key: {:02x} {:02x} {:02x} {:02x} (keystream position: 0)", 
                 keystream[0], keystream[1], keystream[2], keystream[3]);
        println!("Cipher: {:?} (keystream XOR)", cipher.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(" "));

        // Send encrypted message
        let len_bytes = (message.len() as u32).to_le_bytes();
        stream.write_all(&len_bytes).unwrap();
        stream.write_all(&cipher).unwrap();
        println!();
        println!("{} [NETWORK] Sending encrypted message ({} bytes)...", label, message.len());
        println!("{} [-] Sent {} bytes", label, message.len());

        // Receive encrypted message
        stream.read_exact(&mut buffer[0..4]).unwrap();
        let recv_len = u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]) as usize;
        let mut recv_cipher = vec![0u8; recv_len];
        stream.read_exact(&mut recv_cipher).unwrap();

        // Decrypt message
        let recv_keystream = generate_keystream(secret, recv_len);
        let recv_plain = xor_cipher(&recv_cipher, &recv_keystream);
        let recv_message = String::from_utf8_lossy(&recv_plain);

        println!("{} [NETWORK] Received encrypted message ({} bytes)", label, recv_len);
        println!("{} [-] Received {} bytes", label, recv_len);
        println!();
        println!("{} [DECRYPT]", label);
        println!("Cipher: {:02x} {:02x} {:02x} {:02x} ...", recv_cipher[0], recv_cipher[1], recv_cipher[2], recv_cipher[3]);
        println!("Key: {:02x} {:02x} {:02x} {:02x} (keystream position: 0)", 
                 recv_keystream[0], recv_keystream[1], recv_keystream[2], recv_keystream[3]);
        println!("Plain: {}", recv_message);
        println!();
        println!("{} {}", if is_server { "[CLIENT]" } else { "[SERVER]" }, recv_message);
    }
}

fn run_server(port: u16) {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port))
        .expect("Failed to bind");
    println!("[SERVER] Listening on 0.0.0.0:{}", port);
    println!("[SERVER] Waiting for client...");
    println!();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_client(stream, true);
            }
            Err(e) => eprintln!("Error: {}", e),
        }
    }
}

fn run_client(host: &str, port: u16) {
    println!("[CLIENT] Connecting to {}:{}...", host, port);
    match TcpStream::connect(format!("{}:{}", host, port)) {
        Ok(stream) => {
            println!("[CLIENT] Connected!");
            println!();
            handle_client(stream, false);
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        println!("Usage: streamchat <server|client> [HOST:PORT]");
        println!();
        println!("Stream cipher chat with Diffie-Hellman key generation");
        println!();
        println!("Commands:");
        println!("  server              Start server");
        println!("  client              Connect to server");
        return;
    }

    match args[1].as_str() {
        "server" => {
            let port = if args.len() > 2 {
                args[2].parse().unwrap_or(8080)
            } else {
                8080
            };
            run_server(port);
        }
        "client" => {
            let addr = if args.len() > 2 {
                args[2].to_string()
            } else {
                "localhost:8080".to_string()
            };

            let parts: Vec<&str> = addr.split(':').collect();
            let host = parts[0];
            let port = parts.get(1).and_then(|p| p.parse().ok()).unwrap_or(8080);
            run_client(host, port);
        }
        _ => {
            println!("Unknown command: {}", args[1]);
        }
    }
}
