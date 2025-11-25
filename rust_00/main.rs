fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    let mut name = String::from("World");
    let mut upper = false;
    let mut repeat = 1;
    let mut i = 1; // Commence à 1 pour ignorer le nom du programme
    
    // Parser les arguments
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                print_help();
                return;
            }
            "--upper" => {
                upper = true;
                i += 1;
            }
            "--repeat" => {
                i += 1;
                if i < args.len() {
                    repeat = args[i].parse().unwrap_or(1);
                    i += 1;
                }
            }
            arg if !arg.starts_with("--") && !arg.starts_with("-") => {
                name = arg.to_string();
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }
    
    // Construire le message
    let mut message = format!("Hello, {}!", name);
    
    if upper {
        message = message.to_uppercase();
    }
    
    // Afficher le message repeat fois
    for _ in 0..repeat {
        println!("{}", message);
    }
}

fn print_help() {
    println!("Usage: hello [OPTIONS] [NAME]");
    println!();
    println!("Arguments:");
    println!("  [NAME]  Name to greet [default: World]");
    println!();
    println!("Options:");
    println!("  --upper     Convert to uppercase");
    println!("  --repeat    Repeat greeting N times [default: 1]");
    println!("  -h, --help  Print help");
}
