use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{Command, exit};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "playground",
    about = "Run a Rust playground file from src/bin/",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// List all available playgrounds
    #[command(alias = "ls")]
    List,

    /// Print version information
    #[command(alias = "v")]
    Version,

    /// Run a playground by name (default when no subcommand given)
    #[command(external_subcommand)]
    Run(Vec<String>),
}

fn list_playgrounds() -> Vec<String> {
    let dir = PathBuf::from("src/bin");
    if !dir.exists() {
        return vec![];
    }
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let path = e.path();
            if path.extension()?.to_str()? == "rs" {
                path.file_stem()?.to_str().map(|s| s.to_string())
            } else {
                None
            }
        })
        .collect();
    names.sort();
    names
}

fn print_list() {
    let playgrounds = list_playgrounds();
    if playgrounds.is_empty() {
        println!("No playgrounds found. Add a .rs file to src/bin/.");
    } else {
        println!("Available playgrounds:\n");
        for name in &playgrounds {
            println!("  {}", name);
        }
    }
}

fn pick_playground() -> String {
    let playgrounds = list_playgrounds();

    if playgrounds.is_empty() {
        eprintln!("No playgrounds found. Add a .rs file to src/bin/.");
        exit(1);
    }

    println!("Available playgrounds:\n");
    for (i, name) in playgrounds.iter().enumerate() {
        println!("  [{}] {}", i + 1, name);
    }

    loop {
        print!("\nPick a playground: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if let Ok(n) = input.parse::<usize>() {
            if n >= 1 && n <= playgrounds.len() {
                return playgrounds[n - 1].clone();
            }
        } else if playgrounds.contains(&input.to_string()) {
            return input.to_string();
        }

        eprintln!("Invalid choice. Enter a number (1–{}) or a name.", playgrounds.len());
    }
}

fn run_playground(name: &str, args: &[String]) {
    let bin_path = PathBuf::from("src/bin").join(format!("{}.rs", name));
    if !bin_path.exists() {
        eprintln!("Error: playground `{}` not found (looked for {})", name, bin_path.display());
        exit(1);
    }

    let status = Command::new("cargo")
        .args(["run", "--bin", name, "--"])
        .args(args)
        .status()
        .expect("failed to invoke cargo");

    exit(status.code().unwrap_or(0));
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        None => {
            let name = pick_playground();
            run_playground(&name, &[]);
        }
        Some(Commands::List) => print_list(),
        Some(Commands::Version) => {
            println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        }
        Some(Commands::Run(args)) => {
            run_playground(&args[0], &args[1..]);
        }
    }
}
