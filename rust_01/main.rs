use clap::Parser;
use std::collections::HashMap;

/// Compteur de fréquence des mots avec HashMap et itérateurs
#[derive(Parser, Debug)]
#[command(name = "wordfreq", version = "1.0", about = "Count word frequency in text")]
struct Args {
    /// Text to analyze (or use stdin)
    text: Option<String>,

    /// Show top N words [default: 10]
    #[arg(short, long, default_value_t = 10)]
    top: usize,

    /// Ignore words shorter than N [default: 1]
    #[arg(short, long, default_value_t = 1)]
    min_length: usize,

    /// Ignore case
    #[arg(long)]
    ignore_case: bool,

    /// Help
    #[arg(short, long)]
    help: bool,
}

fn main() {
    let args = Args::parse();

    if args.help {
        println!("Usage: wordfreq [OPTIONS] [TEXT]");
        println!();
        println!("Count word frequency in text");
        println!();
        println!("Arguments:");
        println!("  Text to analyze (or use stdin)");
        println!();
        println!("Options:");
        println!("  --top N             Show top N words [default: 10]");
        println!("  --min-length N      Ignore words shorter than N [default: 1]");
        println!("  --ignore-case       Ignore case");
        println!("  -h, --help          Print help");
        return;
    }

    // Get input text
    let text = if let Some(t) = args.text {
        t
    } else {
        // Read from stdin
        use std::io::{self, Read};
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer).expect("Failed to read stdin");
        buffer
    };

    // Count word frequencies using HashMap
    let mut word_freq: HashMap<String, u32> = HashMap::new();

    for word in text.split_whitespace() {
        // Clean punctuation
        let word = word
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect::<String>();

        if word.len() < args.min_length {
            continue;
        }

        let word_key = if args.ignore_case {
            word.to_lowercase()
        } else {
            word
        };

        *word_freq.entry(word_key).or_insert(0) += 1;
    }

    // Sort by frequency and collect top N
    let mut freq_vec: Vec<_> = word_freq.iter().collect();
    freq_vec.sort_by(|a, b| b.1.cmp(a.1));

    // Print results
    println!("Word frequency:");
    for (word, count) in freq_vec.iter().take(args.top) {
        println!("{}: {}", word, count);
    }
}
