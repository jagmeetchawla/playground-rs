# playground-rs

A minimal Rust CLI for running one-off experiments. Drop any `.rs` file into `src/bin/`, run it instantly — no setup, no boilerplate.

## How it works

Every file in `src/bin/` is a standalone Cargo binary target. The `playground` runner lets you discover and run them without remembering `--bin` flags. Each playground is a self-contained `fn main()` with access to every dependency declared in `Cargo.toml`.

## Usage

```sh
# Interactive — lists all playgrounds and asks you to pick one
cargo run

# Run a specific playground
cargo run hello
cargo run my_experiment

# Pass arguments through to the playground
cargo run my_experiment -- --foo bar

# List all available playgrounds
cargo run list
cargo run ls

# Version
cargo run -- --version
```

## Adding a playground

Create a `.rs` file in `src/bin/`:

```sh
touch src/bin/my_experiment.rs
```

```rust
// src/bin/my_experiment.rs

fn main() {
    println!("hello from my experiment");
}
```

Then run it:

```sh
cargo run my_experiment
```

That's it. No registration, no config, no `mod` declarations needed. Cargo auto-discovers every file in `src/bin/` as a binary target.

## Adding dependencies

Add them to `Cargo.toml` as usual — they're available to every playground:

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
rand = "0.8"
```

```rust
// src/bin/json_play.rs
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct Point { x: f64, y: f64 }

fn main() {
    let p = Point { x: 1.0, y: 2.0 };
    println!("{:?}", p);
}
```

## Project structure

```
src/
  main.rs          ← the playground runner (list, pick, delegate)
  bin/
    hello.rs       ← example playground
    <your_file>.rs ← add as many as you want
Cargo.toml         ← shared dependencies for all playgrounds
```

## Requirements

- Rust + Cargo ([rustup.rs](https://rustup.rs))

## License

MIT
