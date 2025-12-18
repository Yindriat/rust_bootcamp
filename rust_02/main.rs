use clap::Parser;
use std::fs;
use std::io::{self, Read, Write, Seek, SeekFrom};

/// Outil hexadécimal pour lire et écrire des fichiers binaires
#[derive(Parser, Debug)]
#[command(name = "hextool", version = "1.0", about = "Read & Write binary files in hexadecimal")]
struct Args {
    /// Fichier cible
    #[arg(short, long)]
    file: String,

    /// Mode lecture (affiche hex)
    #[arg(short, long)]
    read: bool,

    /// Mode écriture (hex string à écrire)
    #[arg(short, long)]
    write: Option<String>,

    /// Offset en bytes (décimal ou hex avec 0x)
    #[arg(short, long, default_value = "0")]
    offset: String,

    /// Nombre de bytes à lire
    #[arg(short, long)]
    size: Option<usize>,

    /// Aide
    #[arg(short, long)]
    help: bool,
}

fn parse_offset(offset_str: &str) -> Result<u64, String> {
    if offset_str.starts_with("0x") || offset_str.starts_with("0X") {
        u64::from_str_radix(&offset_str[2..], 16)
            .map_err(|_| format!("Invalid hex offset: {}", offset_str))
    } else {
        offset_str
            .parse::<u64>()
            .map_err(|_| format!("Invalid offset: {}", offset_str))
    }
}

fn hex_string_to_bytes(hex_str: &str) -> Result<Vec<u8>, String> {
    let hex_str = hex_str.replace(" ", "");
    if hex_str.len() % 2 != 0 {
        return Err("Hex string must have even length".to_string());
    }

    (0..hex_str.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex_str[i..i + 2], 16)
                .map_err(|_| format!("Invalid hex byte: {}", &hex_str[i..i + 2]))
        })
        .collect()
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(" ")
}

fn is_printable(b: u8) -> char {
    if b >= 32 && b < 127 {
        b as char
    } else {
        '.'
    }
}

fn read_binary_file(filename: &str, offset: u64, size: Option<usize>) -> Result<(), String> {
    let mut file = fs::File::open(filename)
        .map_err(|e| format!("Cannot open file: {}", e))?;

    file.seek(SeekFrom::Start(offset))
        .map_err(|e| format!("Cannot seek: {}", e))?;

    let metadata = fs::metadata(filename)
        .map_err(|e| format!("Cannot get file info: {}", e))?;
    let file_size = metadata.len();

    let bytes_to_read = if let Some(s) = size {
        s.min((file_size - offset) as usize)
    } else {
        (file_size - offset) as usize
    };

    let mut buffer = vec![0u8; bytes_to_read];
    file.read_exact(&mut buffer)
        .map_err(|e| format!("Cannot read file: {}", e))?;

    // Display hex dump
    for (i, chunk) in buffer.chunks(16).enumerate() {
        let chunk_offset = offset + (i * 16) as u64;
        let hex_str = bytes_to_hex(chunk);
        let ascii_str: String = chunk.iter().map(|&b| is_printable(b)).collect();
        println!("{:08x}: {:47}  |{}|", chunk_offset, hex_str, ascii_str);
    }

    Ok(())
}

fn write_binary_file(filename: &str, offset: u64, hex_data: &str) -> Result<(), String> {
    let bytes = hex_string_to_bytes(hex_data)?;

    let mut file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(filename)
        .map_err(|e| format!("Cannot open file: {}", e))?;

    file.seek(SeekFrom::Start(offset))
        .map_err(|e| format!("Cannot seek: {}", e))?;

    file.write_all(&bytes)
        .map_err(|e| format!("Cannot write: {}", e))?;

    println!("Writing {} bytes at offset 0x{:x}", bytes.len(), offset);
    println!("Hex: {}", bytes_to_hex(&bytes));
    println!("ASCII: {}", bytes.iter().map(|&b| is_printable(b)).collect::<String>());
    println!("✓ Successfully written");

    Ok(())
}

fn main() {
    let args = Args::parse();

    if args.help {
        println!("Usage: hextool [OPTIONS]");
        println!();
        println!("Read & Write binary files in hexadecimal");
        println!();
        println!("Options:");
        println!("  -f, --file <FILE>       Target file");
        println!("  -r, --read              Read mode (display hex)");
        println!("  -w, --write <HEX>       Write mode (hex string to write)");
        println!("  -o, --offset <OFFSET>   Offset in bytes (decimal or 0x...)");
        println!("  -s, --size <SIZE>       Number of bytes to read");
        println!("  -h, --help              Print help");
        return;
    }

    let offset = match parse_offset(&args.offset) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("Error: {}", e);
            return;
        }
    };

    if let Some(hex_data) = args.write {
        if let Err(e) = write_binary_file(&args.file, offset, &hex_data) {
            eprintln!("Error: {}", e);
        }
    } else if args.read {
        if let Err(e) = read_binary_file(&args.file, offset, args.size) {
            eprintln!("Error: {}", e);
        }
    } else {
        eprintln!("Please specify --read or --write mode");
    }
}
