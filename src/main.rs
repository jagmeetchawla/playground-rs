macro_rules! playgrounds {
    ( $( $name:ident ),* ) => {
        $( paste::paste!{ mod [<$name _playground>]; } )*

        use clap::Parser;
        use clap::builder::{PossibleValuesParser, TypedValueParser};

        #[derive(Parser)]
        #[command(name = "playground", version, about = "Run Rust playground modules")]
        struct Cli {
            /// Playground module to run
            #[arg(
                value_name = "MODULE",
                value_parser = PossibleValuesParser::new([ $( stringify!($name), )* ]).map(|s| s.to_string()),
            )]
            module: Option<String>,
        }

        fn main() {
            let cli = Cli::parse();
            match cli.module.as_deref() {
                None => {
                    println!("Usage: cargo run -- <MODULE>\n\nAvailable modules:");
                    $( println!("  {}", stringify!($name)); )*
                }
                Some(module) => match module {
                    $( stringify!($name) => paste::paste!{ [<$name _playground>]::run() }, )*
                    _ => unreachable!(),
                },
            }
        }
    };
}

include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/_playgrounds.rs"));
