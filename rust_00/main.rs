use clap::Parser;

/// Rusty Hello - CLI arguments et ownership
#[derive(Parser, Debug)]
#[command(name = "hello", version = "1.0", about = "Génère des salutations")]
struct Args {
    /// Name to greet
    #[arg(default_value = "World")]
    name: String,

    /// Convert to uppercase
    #[arg(short, long)]
    upper: bool,

    /// Repeat greeting N times
    #[arg(short, long, default_value_t = 1)]
    repeat: u8,
}

fn main() {
    let args = Args::parse();

    // On prépare le message
    let mut greeting = format!("Hello, {}!", args.name);

    // Manipulation de la String (Ownership/Mutabilité)
    if args.upper {
        greeting = greeting.to_uppercase();
    }

    // Répétition
    for _ in 0..args.repeat {
        println!("{}", greeting);
    }
}