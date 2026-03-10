macro_rules! playgrounds {
    ( $( $name:ident ),* ) => {
        $( paste::paste!{ mod [<$name _playground>]; } )*

        use std::env;

        fn main() {
            let args: Vec<String> = env::args().collect();
            let module = args.get(1).map(String::as_str).unwrap_or("");

            match module {
                $( stringify!($name) => paste::paste!{ [<$name _playground>]::run() }, )*
                "" => {
                    println!("Usage: cargo run -- <module>\n\nAvailable modules:");
                    $( println!("  {}", stringify!($name)); )*
                }
                other => eprintln!("Unknown module: '{other}'"),
            }
        }
    };
}

include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/_playgrounds.rs"));
