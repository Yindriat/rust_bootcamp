use clap::Parser;
use rand::Rng;
use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;
use std::fs;
use std::io::Write;

#[derive(Parser, Debug)]
#[command(name = "hexpath", version = "1.0", about = "Find min/max cost paths in hexadecimal grid")]
struct Args {
    /// Map file (hex values, space separated)
    map_file: Option<String>,

    /// Generate random map (e.g., 8x4, 16x12)
    #[arg(long)]
    generate: Option<String>,

    /// Output save generated map to file
    #[arg(long)]
    output: Option<String>,

    /// Visualize Show colored map
    #[arg(long)]
    visualize: bool,

    /// Both Show both min and max paths
    #[arg(long)]
    both: bool,

    /// Animate ANIMATE pathfinding
    #[arg(long)]
    animate: bool,

    /// Help
    #[arg(short, long)]
    help: bool,
}

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
struct Pos(usize, usize);

#[derive(Clone, Copy, Eq, PartialEq)]
struct State {
    cost: u32,
    pos: Pos,
    find_max: bool,
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.find_max {
            other.cost.cmp(&self.cost)
        } else {
            other.cost.cmp(&self.cost).reverse()
        }
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

struct Grid {
    cells: Vec<Vec<u8>>,
    width: usize,
    height: usize,
}

impl Grid {
    fn new(width: usize, height: usize) -> Self {
        Grid {
            cells: vec![vec![0; width]; height],
            width,
            height,
        }
    }

    fn generate_random(width: usize, height: usize) -> Self {
        let mut grid = Grid::new(width, height);
        let mut rng = rand::thread_rng();
        
        for row in &mut grid.cells {
            for cell in row {
                *cell = rng.gen_range(0..=0xFF);
            }
        }
        
        grid.cells[0][0] = 0x00;
        grid.cells[height - 1][width - 1] = 0xFF;
        
        grid
    }

    fn from_file(filename: &str) -> Result<Self, String> {
        let content = fs::read_to_string(filename)
            .map_err(|e| format!("Cannot read file: {}", e))?;

        let mut grid = Grid::new(0, 0);
        let mut rows = vec![];

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let mut row = vec![];
            for hex_str in line.split_whitespace() {
                let byte = u8::from_str_radix(hex_str, 16)
                    .map_err(|_| format!("Invalid hex byte: {}", hex_str))?;
                row.push(byte);
            }

            if !row.is_empty() {
                rows.push(row);
            }
        }

        if rows.is_empty() {
            return Err("Empty map".to_string());
        }

        grid.height = rows.len();
        grid.width = rows[0].len();
        grid.cells = rows;

        Ok(grid)
    }

    fn save(&self, filename: &str) -> Result<(), String> {
        let mut file = fs::File::create(filename)
            .map_err(|e| format!("Cannot create file: {}", e))?;

        for row in &self.cells {
            let hex_row: Vec<String> = row.iter().map(|b| format!("{:02X}", b)).collect();
            writeln!(file, "{}", hex_row.join(" "))
                .map_err(|e| format!("Cannot write file: {}", e))?;
        }

        Ok(())
    }

    fn get(&self, pos: Pos) -> Option<u8> {
        if pos.0 < self.height && pos.1 < self.width {
            Some(self.cells[pos.0][pos.1])
        } else {
            None
        }
    }

    fn neighbors(&self, pos: Pos) -> Vec<Pos> {
        let mut neighbors = vec![];
        let (row, col) = (pos.0, pos.1);

        let directions = [
            (row.saturating_sub(1), col),
            (row + 1, col),
            (row, col.saturating_sub(1)),
            (row, col + 1),
        ];

        for (r, c) in &directions {
            if *r < self.height && *c < self.width {
                neighbors.push(Pos(*r, *c));
            }
        }

        neighbors
    }

    fn dijkstra(&self, find_max: bool, animate: bool) -> (Vec<Pos>, u32) {
        let start = Pos(0, 0);
        let end = Pos(self.height - 1, self.width - 1);

        let mut dist: HashMap<Pos, u32> = HashMap::new();
        let mut parent: HashMap<Pos, Pos> = HashMap::new();
        let mut heap = BinaryHeap::new();

        dist.insert(start, 0);
        heap.push(State {
            cost: 0,
            pos: start,
            find_max,
        });

        let mut step = 0;

        while let Some(State { cost, pos, find_max: _ }) = heap.pop() {
            if animate {
                println!("[>] [Step {}] Exploring ({},{}) - cost: {}", step, pos.0, pos.1, cost);
                step += 1;
            }

            if pos == end {
                let mut path = vec![pos];
                let mut current = pos;

                while let Some(&prev) = parent.get(&current) {
                    path.push(prev);
                    current = prev;
                }

                path.reverse();
                return (path, cost);
            }

            if let Some(&d) = dist.get(&pos) {
                if cost > d {
                    continue;
                }
            }

            for next_pos in self.neighbors(pos) {
                if let Some(next_cost_byte) = self.get(next_pos) {
                    let next_cost_byte = next_cost_byte as u32;
                    let new_cost = if find_max {
                        cost.max(next_cost_byte)
                    } else {
                        cost + next_cost_byte
                    };

                    let should_update = if let Some(&curr_dist) = dist.get(&next_pos) {
                        if find_max {
                            new_cost < curr_dist
                        } else {
                            new_cost < curr_dist
                        }
                    } else {
                        true
                    };

                    if should_update {
                        dist.insert(next_pos, new_cost);
                        parent.insert(next_pos, pos);
                        heap.push(State {
                            cost: new_cost,
                            pos: next_pos,
                            find_max,
                        });
                    }
                }
            }
        }

        (vec![], 0)
    }

    fn print_map(&self) {
        for row in &self.cells {
            let hex_row: Vec<String> = row.iter().map(|b| format!("{:02X}", b)).collect();
            println!("{}", hex_row.join(" "));
        }
    }

    fn visualize(&self, min_path: &[Pos], max_path: &[Pos]) {
        println!("HEXADECIMAL GRID (rainbow gradient):");
        let colors = vec![
            "\x1b[38;5;196m", // Red
            "\x1b[38;5;208m", // Orange
            "\x1b[38;5;226m", // Yellow
            "\x1b[38;5;46m",  // Green
            "\x1b[38;5;51m",  // Cyan
            "\x1b[38;5;21m",  // Blue
            "\x1b[38;5;93m",  // Purple
        ];

        println!();
        for (row_idx, row) in self.cells.iter().enumerate() {
            for (col_idx, &cell) in row.iter().enumerate() {
                let pos = Pos(row_idx, col_idx);
                let color_idx = (cell as usize / 256) * colors.len() / 256;
                let color = colors[color_idx];

                if min_path.contains(&pos) {
                    print!("{} {:02X}\x1b[0m", "\x1b[37m", cell); // White for min
                } else if max_path.contains(&pos) {
                    print!("{} {:02X}\x1b[0m", "\x1b[91m", cell); // Bright red for max
                } else {
                    print!("{}{:02X}\x1b[0m ", color, cell);
                }
            }
            println!();
        }
    }
}

fn print_path_info(label: &str, path: &[Pos], cost: u32) {
    println!();
    println!("{} COST PATH:", label);
    println!("==================");
    println!("Total cost: 0x{:X} ({} decimal)", cost, cost);
    println!("Path length: {} steps", path.len());
    println!("Path:");
    println!("{}", path.iter()
        .map(|p| format!("({},{})", p.0, p.1))
        .collect::<Vec<_>>()
        .join("-"));

    println!();
    println!("Step-by-step costs:");
    for (i, pos) in path.iter().enumerate() {
        let step_num = i;
        println!("- 0x{:02X} ({},{}) +{}", pos.1 * 16 + pos.0, pos.0, pos.1, 
                 if i == 0 { 0 } else { 1 });
    }
    println!("Total: 0x{:X} ({})", cost, cost);
}

fn main() {
    let args = Args::parse();

    if args.help {
        println!("Usage: hexpath [OPTIONS] [MAP_FILE]");
        println!();
        println!("Find min/max cost paths in hexadecimal grid");
        println!();
        println!("Arguments:");
        println!("  Map file (hex values, space separated)");
        println!();
        println!("Options:");
        println!("  --generate WxH          Generate random map (e.g., 8x4, 10x10)");
        println!("  --output FILE           Save generated map to file");
        println!("  --visualize             Show colored map");
        println!("  --both                  Show both min and max paths");
        println!("  --animate               Animate pathfinding");
        println!("  -h, --help              Print help");
        return;
    }

    let mut grid = if let Some(gen_spec) = args.generate {
        let parts: Vec<&str> = gen_spec.split('x').collect();
        if parts.len() != 2 {
            eprintln!("Error: Invalid generate format. Use WxH (e.g., 8x4)");
            return;
        }

        let width: usize = parts[0].parse().unwrap_or(8);
        let height: usize = parts[1].parse().unwrap_or(4);

        println!("Generating {}x{} hexadecimal grid...", width, height);
        let generated = Grid::generate_random(width, height);

        if let Some(output_file) = args.output {
            if let Err(e) = generated.save(&output_file) {
                eprintln!("Error: {}", e);
                return;
            }
            println!("Map saved to: {}", output_file);
        }

        println!();
        println!("Generated map:");
        generated.print_map();

        generated
    } else if let Some(map_file) = args.map_file {
        match Grid::from_file(&map_file) {
            Ok(g) => g,
            Err(e) => {
                eprintln!("Error: {}", e);
                return;
            }
        }
    } else {
        eprintln!("Error: Provide map file or use --generate");
        return;
    };

    println!();
    println!("Analyzing hexadecimal grid...");
    println!("Grid size: {}x{}", grid.width, grid.height);
    println!("Start: (0,0) = 0x00");
    println!("End: ({},{}) = 0x{:02X}", grid.height - 1, grid.width - 1, 
             grid.cells[grid.height - 1][grid.width - 1]);

    if args.animate {
        println!();
        println!("Searching for minimum cost path...");
    }

    let (min_path, min_cost) = grid.dijkstra(false, args.animate);

    if args.both {
        if args.animate {
            println!();
            println!("Searching for maximum cost path...");
        }
        let (max_path, max_cost) = grid.dijkstra(true, args.animate);

        print_path_info("MINIMUM", &min_path, min_cost);
        print_path_info("MAXIMUM", &max_path, max_cost);

        if args.visualize {
            println!();
            grid.visualize(&min_path, &max_path);
        }
    } else {
        print_path_info("MINIMUM", &min_path, min_cost);

        if args.visualize {
            println!();
            grid.visualize(&min_path, &[]);
        }
    }
}
