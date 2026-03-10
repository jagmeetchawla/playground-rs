# playground-rs

A Rust playground for experimenting with code. Add a module, run it by name — no boilerplate.

## Usage

```sh
# List available playgrounds
cargo run

# Run a specific playground
cargo run -- variables
```

## Adding a new playground

1. Create `src/<name>_playground.rs` with a `pub fn run()` inside it:

```rust
pub fn run() {
    // your experiments here
}
```

2. That's it. `cargo run` will pick it up automatically.

---

## How it works

### The build script — `build.rs`

Cargo has a convention: if `build.rs` exists at the project root, Cargo compiles and runs it *before* compiling your actual code. It's just a regular Rust program that executes at build time.

`build.rs` scans `src/`, finds every file ending in `_playground.rs`, strips the suffix to get the bare names (e.g. `variables`), sorts them, then writes a generated file `src/_playgrounds.rs`:

```rust
// @generated — do not edit, updated by build.rs
playgrounds!(variables);
```

The line `println!("cargo:rerun-if-changed=src/")` is a message to Cargo via stdout. It tells Cargo to only re-run `build.rs` when something in `src/` changes, rather than on every build.

---

### `_playgrounds.rs` gets inlined into `main.rs`

The last line of `main.rs`:

```rust
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/_playgrounds.rs"));
```

- `env!("CARGO_MANIFEST_DIR")` — compile-time macro that expands to the project root path
- `concat!(...)` — joins it with the filename into a single string, also at compile time
- `include!(...)` — pastes the file contents verbatim, as if you had typed it yourself

After this, the compiler sees `playgrounds!(variables);` at the bottom of `main.rs`.

---

### The `playgrounds!` macro expands

The macro takes the list of names and generates three things for each one. The pattern `$( ... ),*` means *"repeat for each item in the comma-separated list"*.

**a) Module declarations**

```rust
$( paste::paste!{ mod [<$name _playground>]; } )*
```

`paste!` is needed because Rust macros can't concatenate identifiers natively. `[<variables _playground>]` becomes `variables_playground`. This generates:

```rust
mod variables_playground;
```

This tells the compiler: find `src/variables_playground.rs` and compile it as a module.

**b) Match arms**

```rust
$( stringify!($name) => paste::paste!{ [<$name _playground>]::run() }, )*
```

`stringify!` turns the identifier `variables` into the string literal `"variables"`. This generates:

```rust
"variables" => variables_playground::run(),
```

**c) The listing**

```rust
$( println!("  {}", stringify!($name)); )*
```

Generates a `println!` for each name, used when no argument is passed.

---

### The full picture

```
cargo run
    │
    ├─ build.rs runs
    │       scans src/ → finds variables_playground.rs
    │       writes src/_playgrounds.rs:  playgrounds!(variables);
    │
    ├─ main.rs compiles
    │       include!() pastes _playgrounds.rs in
    │       playgrounds!(variables) macro expands into:
    │           mod variables_playground;
    │           fn main() { match ... "variables" => variables_playground::run() ... }
    │
    └─ binary runs
            cargo run           →  lists available modules
            cargo run -- variables  →  variables_playground::run()
```

The key constraint: **Rust resolves all module names and function calls at compile time** — there's no runtime reflection. The build script + macro combination is the standard Rust pattern to keep the developer experience fully automatic while satisfying that constraint.
