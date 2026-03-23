macro_rules! playgrounds {
    ( $( $name:ident ),* ) => {
        $( paste::paste!{ mod [<$name _playground>]; } )*

        use clap::{CommandFactory, Parser};

        #[derive(Clone)]
        enum Command {
            Version,
            Help,
            List,
            Module(String),
        }

        fn parse_command(s: &str) -> Result<Command, Box<dyn std::error::Error + Send + Sync>> {
            match s {
                "version" | "v" => Ok(Command::Version),
                "help" | "h" => Ok(Command::Help),
                "list" => Ok(Command::List),
                $( stringify!($name) => Ok(Command::Module(s.to_string())), )*
                _ => Err(format!("unknown command or module '{s}'").into()),
            }
        }

        fn print_playground_list() {
            println!("Available modules:");
            $( println!("  {}", stringify!($name)); )*
        }

        #[derive(Parser)]
        #[command(name = "playground", version, about = "Run Rust playground modules")]
        struct Cli {
            /// List playground modules and exit
            #[arg(short = 'l', long = "list", conflicts_with = "command")]
            list: bool,
            /// Playground module to run, or `version` / `v`, `help` / `h`, `list`
            #[arg(value_name = "COMMAND", value_parser = parse_command)]
            command: Option<Command>,
        }

        fn main() {
            let cli = Cli::parse();
            if cli.list {
                print_playground_list();
                return;
            }
            match cli.command {
                None => {
                    println!("Usage: cargo run -- <COMMAND>\n\nCommands:");
                    println!("  version, v   Print the program version");
                    println!("  help, h      Print command-line help");
                    println!("  list         List available playground modules");
                    println!("  -l, --list   Same as `list`");
                    println!("\nAvailable modules:");
                    $( println!("  {}", stringify!($name)); )*
                }
                Some(Command::Version) => {
                    println!("{}", env!("CARGO_PKG_VERSION"));
                }
                Some(Command::Help) => {
                    Cli::command().print_help().expect("write help to stdout");
                    println!();
                }
                Some(Command::List) => {
                    print_playground_list();
                }
                Some(Command::Module(name)) => match name.as_str() {
                    $( stringify!($name) => paste::paste!{ [<$name _playground>]::run() }, )*
                    _ => unreachable!(),
                },
            }
        }
    };
}

include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/_playgrounds.rs"));
