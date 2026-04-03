use tauri::AppHandle;

#[tauri::command]
pub fn seed_rust_book(app: AppHandle) -> Result<Vec<String>, String> {
    let pdir = crate::projects_dir(&app);
    let mut created: Vec<String> = Vec::new();

    // Attribution text placed in every chapter's content/ folder.
    // "The Rust Programming Language" is © Rust Project Developers (2010),
    // dual-licensed MIT / Apache-2.0.  https://github.com/rust-lang/book
    const ATTRIBUTION_MD: &str = "\
# Source: The Rust Programming Language

These playground examples are based on the curriculum of:

**\"The Rust Programming Language\"**
by Steve Klabnik and Carol Nichols,
with contributions from the Rust Community.

| | |
|---|---|
| Book (free online) | https://doc.rust-lang.org/book/ |
| Source repository  | https://github.com/rust-lang/book |
| License            | MIT / Apache-2.0 |
| Copyright          | Rust Project Developers (2010) |

## Notes

The playground code is original educational Rust written to illustrate the
same concepts covered in each chapter.  It is not verbatim source from the
book.  Rustic Playground is not affiliated with or endorsed by the Rust
Project or its contributors.
";

    // (project_name, extra_cargo_deps, [(playground_name, code)])
    #[allow(clippy::type_complexity)]
    let chapters: Vec<(&str, &str, Vec<(&str, &str)>)> = vec![
        // ── Chapter 1: Getting Started ────────────────────────────────────────
        (
            "ch01_getting_started",
            "",
            vec![
                (
                    "hello",
                    r#"// Ch 1 – Hello, World!
// Every Rust program starts with fn main(). println! is a macro — note the !

fn main() {
    println!("Hello, world!");

    // Format strings use {} as a placeholder
    let language = "Rust";
    let year: u32 = 2015;
    println!("{language} reached 1.0 in {year}.");

    // Multiple arguments
    println!("{} + {} = {}", 1, 2, 1 + 2);

    // Padding & alignment
    println!("{:>10}", "right");   // right-aligned in 10 chars
    println!("{:<10}", "left");    // left-aligned
    println!("{:^10}", "centre");  // centred

    // Debug formatting with {:?}
    let coords = (3.0_f64, -7.5_f64);
    println!("Coordinates: {:?}", coords);
    println!("Pretty:      {:#?}", coords);
}
"#,
                ),
                (
                    "variables",
                    r#"// Ch 1/3 – Variables, Mutability, Shadowing, Constants
// Variables are immutable by default — a key Rust safety feature.

const MAX_SCORE: u32 = 100_000; // constants always require a type annotation

fn main() {
    // Immutable binding (default)
    let x = 5;
    println!("x = {x}");
    // x = 6; // ← compile error: cannot assign twice to immutable variable

    // Mutable binding
    let mut count = 0;
    count += 1;
    count += 1;
    println!("count = {count}");

    // Shadowing: rebind the same name — even to a different type
    let text = "hello world";
    let text = text.len(); // text is now usize, not &str
    println!("text length = {text}");

    // Block-scoped shadowing
    let val = 10;
    let val = {
        let val = val * 2; // inner shadow
        val + 5            // block evaluates to 25
    };
    println!("val = {val}"); // 25

    println!("MAX_SCORE = {MAX_SCORE}");
}
"#,
                ),
            ],
        ),
        // ── Chapter 2: Guessing Game ──────────────────────────────────────────
        (
            "ch02_guessing_game",
            "rand = \"0.8\"\n",
            vec![(
                "guessing_game",
                r#"// Ch 2 – Guessing Game
// Introduces: external crates (rand), Ordering, match, loop, type conversion.
//
// Note: stdin is not available in this playground, so "user guesses" are
// simulated with a hardcoded array.  Try changing the values!

use rand::Rng;
use std::cmp::Ordering;

fn main() {
    let secret: u32 = rand::thread_rng().gen_range(1..=100);
    println!("=== Guessing Game ===");
    println!("(Secret revealed for demo: {secret})\n");

    // In the book you would read with: io::stdin().read_line(&mut guess)
    // Here we simulate a binary-search strategy:
    let guesses = [50u32, 25, 37, 43, 47, 49, secret];

    for guess in guesses {
        if !(1..=100).contains(&guess) {
            println!("Guess {guess} is out of range 1–100, skipping.");
            continue;
        }
        println!("You guessed: {guess}");
        match guess.cmp(&secret) {
            Ordering::Less    => println!("  → Too small!"),
            Ordering::Greater => println!("  → Too big!"),
            Ordering::Equal   => {
                println!("  → You win! 🎉");
                return;
            }
        }
    }
    println!("Ran out of guesses — secret was {secret}.");
}
"#,
            )],
        ),
        // ── Chapter 3: Common Programming Concepts ────────────────────────────
        (
            "ch03_concepts",
            "",
            vec![
                (
                    "data_types",
                    r#"// Ch 3 – Data Types
// Rust is statically typed — every value has a type known at compile time.

fn main() {
    // ── Scalar types ──────────────────────────────────────────────────────────
    let a: i32  = -1_000;
    let b: u64  = 1_000_000;
    let c: u8   = 0b1010_1010; // binary literal
    let d: u16  = 0xff;        // hex literal
    println!("ints: {a}, {b}, {c}, {d}");

    let pi: f64 = 3.141_592_653_589_793;
    let e:  f32 = 2.718_28;
    println!("floats: {pi:.5}, {e:.5}");

    let t: bool = true;
    let f: bool = false;
    println!("bool: {t}, {f}");

    let letter = 'A';
    let emoji  = '\u{1F980}'; // 🦀 Unicode scalar value
    println!("chars: {letter}, {emoji}");

    // ── Compound types ────────────────────────────────────────────────────────

    // Tuple — fixed length, mixed types
    let tup: (i32, f64, bool) = (500, 6.4, true);
    let (x, y, z) = tup;         // destructuring
    println!("tuple destructured: {x}, {y}, {z}");
    println!("index access: {}", tup.0);

    // Array — fixed length, same type, stack-allocated
    let arr = [1, 2, 3, 4, 5];
    println!("arr[0] = {}", arr[0]);
    println!("arr: {:?}", arr);

    // Array with repeated value: [value; count]
    let zeros = [0u8; 8];
    println!("zeros: {:?}", zeros);
}
"#,
                ),
                (
                    "functions",
                    r#"// Ch 3 – Functions
// Functions are declared with fn. Return type follows ->.
// The last expression (no semicolon) in a block is its return value.

fn add(x: i32, y: i32) -> i32 {
    x + y  // expression — no semicolon — is the return value
}

fn five() -> i32 { 5 }

fn plus_one(x: i32) -> i32 { x + 1 }

fn temperature_label(c: f64) -> &'static str {
    if      c < 0.0  { "freezing" }
    else if c < 15.0 { "cold" }
    else if c < 25.0 { "comfortable" }
    else             { "hot" }
}

fn print_measurement(value: i32, unit: char) {
    println!("Measurement: {value}{unit}");
}

fn main() {
    println!("add(5, 6)    = {}", add(5, 6));
    println!("five()       = {}", five());
    println!("plus_one(5)  = {}", plus_one(5));
    println!("30°C is      {}", temperature_label(30.0));
    print_measurement(42, 'm');

    // Blocks are expressions — the last expression is the block's value
    let y = {
        let x = 3;
        x + 1  // evaluates to 4 — no semicolon!
    };
    println!("block value y = {y}");

    // Statements return () — adding a semicolon turns an expression into one
    // let z = (let x = 6); // ← error: let is a statement, not an expression
}
"#,
                ),
                (
                    "control_flow",
                    r#"// Ch 3 – Control Flow
// if/else, loop with return value, while, for, loop labels

fn main() {
    // ── if / else if / else ───────────────────────────────────────────────────
    let number = 7;
    if number < 5 {
        println!("{number} < 5");
    } else if number % 2 == 0 {
        println!("{number} is even");
    } else {
        println!("{number} is odd and ≥ 5");
    }

    // if is an expression — use it on the right of let
    let description = if number % 2 == 0 { "even" } else { "odd" };
    println!("{number} is {description}");

    // ── loop ─────────────────────────────────────────────────────────────────
    let mut counter = 0;
    let result = loop {
        counter += 1;
        if counter == 10 {
            break counter * 2; // break can return a value
        }
    };
    println!("loop result = {result}");

    // Nested loops with labels
    'outer: for i in 0..3 {
        for j in 0..3 {
            if i == j { continue 'outer; }
            print!("({i},{j}) ");
        }
    }
    println!();

    // ── while ────────────────────────────────────────────────────────────────
    let mut n = 3;
    while n != 0 { print!("{n} "); n -= 1; }
    println!("LIFTOFF!");

    // ── for + ranges ─────────────────────────────────────────────────────────
    let a = [10, 20, 30, 40, 50];
    for element in a { print!("{element} "); }
    println!();

    for i in (1..=5).rev() { print!("{i} "); }
    println!("🚀");
}
"#,
                ),
            ],
        ),
        // ── Chapter 4: Ownership ──────────────────────────────────────────────
        (
            "ch04_ownership",
            "",
            vec![
                (
                    "ownership",
                    r#"// Ch 4 – Ownership
// Rules: (1) Each value has one owner. (2) Only one owner at a time.
//        (3) Value is dropped when its owner goes out of scope.

fn takes_ownership(s: String) { println!("got: {s}"); } // s dropped here
fn makes_copy(x: i32)         { println!("copy: {x}"); }
fn gives_ownership() -> String { String::from("given") }
fn takes_and_gives_back(s: String) -> String { s }

fn main() {
    // Stack (Copy) types are copied, not moved
    let x = 5;
    let y = x;                    // copies the value
    println!("x={x}, y={y}");    // both valid

    // Heap (non-Copy) types are moved
    let s1 = String::from("hello");
    let s2 = s1;                  // s1 is MOVED — it is no longer valid
    // println!("{s1}");          // ← compile error: value moved
    println!("s2 = {s2}");

    // Clone makes a deep copy
    let s3 = String::from("world");
    let s4 = s3.clone();
    println!("s3={s3}, s4={s4}"); // both valid

    // Ownership and functions
    let s = String::from("ownership");
    takes_ownership(s);
    // println!("{s}");           // ← error: s moved into function

    let x = 5;
    makes_copy(x);
    println!("x is still {x}");  // fine — i32 is Copy

    // Returning values transfers ownership
    let s1 = gives_ownership();
    let s2 = String::from("hello");
    let s3 = takes_and_gives_back(s2);
    println!("{s1}, {s3}");
}
"#,
                ),
                (
                    "references",
                    r#"// Ch 4 – References & Borrowing
// A reference lets you refer to a value without taking ownership.
// Rules: unlimited &T borrows OR exactly one &mut T borrow, never both.

fn calculate_length(s: &String) -> usize { s.len() }
fn change(s: &mut String) { s.push_str(", world"); }

fn main() {
    // Immutable reference — borrow without taking ownership
    let s1 = String::from("hello");
    let len = calculate_length(&s1);
    println!("'{s1}' has length {len}");  // s1 still valid

    // Mutable reference
    let mut s = String::from("hello");
    change(&mut s);
    println!("after change: {s}");

    // Multiple immutable borrows are fine
    let s = String::from("hello");
    let r1 = &s;
    let r2 = &s;
    println!("{r1}, {r2}");  // r1 and r2 last used here

    // Now we can take a mutable borrow (r1/r2 are out of use)
    let mut s = String::from("hi");
    let r3 = &mut s;
    r3.push_str(" there");
    println!("{r3}");

    // The borrow checker prevents dangling references:
    // fn dangle() -> &String { let s = String::from("x"); &s }
    // ↑ compile error: s is dropped but a reference to it would be returned
}
"#,
                ),
                (
                    "slices",
                    r#"// Ch 4 – Slices
// Slices reference a contiguous sequence without taking ownership.

fn first_word(s: &str) -> &str {
    let bytes = s.as_bytes();
    for (i, &byte) in bytes.iter().enumerate() {
        if byte == b' ' { return &s[..i]; }
    }
    &s[..]
}

fn sum_of_slice(numbers: &[i32]) -> i32 { numbers.iter().sum() }

fn main() {
    // String slices — a reference to part of a String
    let s = String::from("hello world");
    let hello = &s[0..5];  // or &s[..5]
    let world = &s[6..11]; // or &s[6..]
    println!("{hello} {world}");

    let sentence = String::from("learning rust is fun");
    let word = first_word(&sentence);
    println!("First word: {word}");
    // sentence.clear(); // ← compile error: can't mutate while borrowed as &str

    // String literals ARE slices (&str pointing into the binary)
    let literal = "hello, world";
    println!("First word of literal: {}", first_word(literal));

    // Array slices
    let a = [1, 2, 3, 4, 5];
    let slice: &[i32] = &a[1..3];
    println!("slice: {:?}", slice);
    println!("sum of a[2..]: {}", sum_of_slice(&a[2..]));
}
"#,
                ),
            ],
        ),
        // ── Chapter 5: Structs ────────────────────────────────────────────────
        (
            "ch05_structs",
            "",
            vec![
                (
                    "structs",
                    r#"#![allow(dead_code)]
// Ch 5 – Defining and Using Structs

#[derive(Debug)]
struct User {
    username: String,
    email:    String,
    active:   bool,
    sign_in_count: u64,
}

// Tuple structs — named tuple types
struct Color(u8, u8, u8);
struct Point(f64, f64, f64);

// Unit-like struct (no fields — useful when implementing traits)
struct AlwaysEqual;

fn build_user(email: String, username: String) -> User {
    User {
        email,    // field-init shorthand (name matches variable)
        username,
        active: true,
        sign_in_count: 1,
    }
}

fn main() {
    let mut user1 = build_user(
        String::from("alice@example.com"),
        String::from("alice"),
    );
    user1.sign_in_count += 1;
    println!("{:#?}", user1);

    // Struct update syntax — fill remaining fields from another instance
    let user2 = User {
        email:    String::from("bob@example.com"),
        username: String::from("bob"),
        ..user1  // copy active & sign_in_count from user1 (both are Copy)
    };
    println!("user2 active={}", user2.active);

    let red    = Color(255, 0, 0);
    let origin = Point(0.0, 0.0, 0.0);
    println!("red: ({}, {}, {})", red.0, red.1, red.2);
    println!("origin: ({}, {}, {})", origin.0, origin.1, origin.2);

    let _unit = AlwaysEqual;
}
"#,
                ),
                (
                    "methods",
                    r#"// Ch 5 – Method Syntax
// Methods are defined inside impl blocks. &self borrows; &mut self mutates.

#[derive(Debug)]
struct Rectangle { width: f64, height: f64 }

impl Rectangle {
    // Associated function (no self) — called as Rectangle::square(...)
    fn square(size: f64) -> Self { Self { width: size, height: size } }

    fn area(&self)      -> f64  { self.width * self.height }
    fn perimeter(&self) -> f64  { 2.0 * (self.width + self.height) }
    fn is_square(&self) -> bool { (self.width - self.height).abs() < f64::EPSILON }

    fn can_hold(&self, other: &Rectangle) -> bool {
        self.width > other.width && self.height > other.height
    }

    // Takes ownership, returns modified self — builder pattern
    fn scale(mut self, factor: f64) -> Self {
        self.width  *= factor;
        self.height *= factor;
        self
    }
}

fn main() {
    let rect1 = Rectangle { width: 30.0, height: 50.0 };
    let rect2 = Rectangle { width: 10.0, height: 40.0 };
    let sq    = Rectangle::square(25.0);

    println!("{rect1:?}");
    println!("area      = {:.1}", rect1.area());
    println!("perimeter = {:.1}", rect1.perimeter());
    println!("is square?  {}", rect1.is_square());
    println!("rect1 holds rect2? {}", rect1.can_hold(&rect2));
    println!("sq is square? {}", sq.is_square());

    let big = Rectangle { width: 10.0, height: 5.0 }.scale(3.0);
    println!("scaled: {:?}", big);
}
"#,
                ),
                (
                    "debug_display",
                    r#"// Ch 5 – Derived and Custom Traits
// #[derive] auto-implements common traits.
// Implement fmt::Display yourself for custom {} output.

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
struct Point { x: f64, y: f64 }

impl Point {
    fn new(x: f64, y: f64) -> Self { Self { x, y } }
    fn distance(&self, other: &Point) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

fn main() {
    let p1 = Point::new(0.0, 0.0);
    let p2 = Point::new(3.0, 4.0);

    println!("Debug:   {:?}",  p1);   // from #[derive(Debug)]
    println!("Pretty:  {:#?}", p1);
    println!("Display: {}",    p1);   // from impl fmt::Display

    let p3 = p1.clone();              // from #[derive(Clone)]
    println!("Cloned:  {p3}");

    println!("p1 == p3? {}", p1 == p3);  // from #[derive(PartialEq)]
    println!("p1 == p2? {}", p1 == p2);

    println!("Distance from {p1} to {p2}: {:.2}", p1.distance(&p2));
}
"#,
                ),
            ],
        ),
        // ── Chapter 6: Enums and Pattern Matching ─────────────────────────────
        (
            "ch06_enums",
            "",
            vec![
                (
                    "enums",
                    r#"#![allow(dead_code)]
// Ch 6 – Enums
// Enums define a type that can be one of several variants.
// Variants can carry data — replacing many struct patterns.

#[derive(Debug)]
enum IpAddr {
    V4(u8, u8, u8, u8),
    V6(String),
}

#[derive(Debug)]
enum Message {
    Quit,
    Move { x: i32, y: i32 },  // named fields
    Write(String),
    ChangeColor(u8, u8, u8),
}

impl Message {
    fn call(&self) {
        match self {
            Message::Quit              => println!("Quit"),
            Message::Move { x, y }    => println!("Move to ({x},{y})"),
            Message::Write(text)       => println!("Write: {text}"),
            Message::ChangeColor(r,g,b)=> println!("Color: rgb({r},{g},{b})"),
        }
    }
}

fn main() {
    let home     = IpAddr::V4(127, 0, 0, 1);
    let loopback = IpAddr::V6(String::from("::1"));
    println!("{home:?}");
    println!("{loopback:?}");

    let messages = vec![
        Message::Quit,
        Message::Move { x: 10, y: 20 },
        Message::Write(String::from("hello")),
        Message::ChangeColor(255, 128, 0),
    ];
    for m in &messages { m.call(); }

    // Option<T> — Rust's null-safe alternative to null pointers
    let some_number: Option<i32> = Some(42);
    let no_number:   Option<i32> = None;
    println!("some: {some_number:?}");
    println!("value or 0: {}", some_number.unwrap_or(0));
    println!("value or 0: {}", no_number.unwrap_or(0));
}
"#,
                ),
                (
                    "match_expr",
                    r#"// Ch 6 – The match Expression
// match compares against patterns exhaustively — all cases must be covered.

#[derive(Debug)]
enum Coin { Penny, Nickel, Dime, Quarter(String) }

fn value_in_cents(coin: &Coin) -> u32 {
    match coin {
        Coin::Penny              => { println!("Lucky penny!"); 1 }
        Coin::Nickel             => 5,
        Coin::Dime               => 10,
        Coin::Quarter(state)     => { println!("Quarter from {state}!"); 25 }
    }
}

fn plus_one(x: Option<i32>) -> Option<i32> {
    match x {
        None    => None,
        Some(i) => Some(i + 1),
    }
}

fn describe(n: i32) -> &'static str {
    match n {
        0          => "zero",
        1 | 2 | 3  => "small",
        4..=9      => "medium",
        n if n < 0 => "negative",   // match guard
        _          => "large",      // catch-all wildcard
    }
}

fn main() {
    let coins = [
        Coin::Penny,
        Coin::Nickel,
        Coin::Dime,
        Coin::Quarter(String::from("Alaska")),
    ];
    for coin in &coins {
        println!("{coin:?} = {} cents", value_in_cents(coin));
    }

    println!("\nplus_one(Some(5)) = {:?}", plus_one(Some(5)));
    println!("plus_one(None)    = {:?}", plus_one(None));

    for n in [-3, 0, 1, 5, 100] {
        println!("{n:>4} → {}", describe(n));
    }
}
"#,
                ),
                (
                    "if_let",
                    r#"// Ch 6 – if let, while let, let-else
// Concise alternatives to match when you only care about one pattern.

fn main() {
    // if let — handle one variant, ignore the rest
    let config: Option<&str> = Some("debug");
    if let Some(mode) = config {
        println!("Config mode: {mode}");
    } else {
        println!("Using default config");
    }

    // Chained if let / else if let
    let age: Result<u8, _> = "34".parse();
    if let Ok(a) = age {
        if a > 30 { println!("Over 30 — use purple!"); }
        else      { println!("30 or under — use orange!"); }
    }

    // while let — keep looping while pattern matches
    let mut stack = vec![1, 2, 3];
    while let Some(top) = stack.pop() {
        print!("{top} ");
    }
    println!("(stack empty)");

    // Matching tuples
    let pair = (true, 42);
    if let (true, n) = pair {
        println!("Got true with {n}");
    }

    // let-else (Rust 1.65+) — must diverge in else branch
    let text = "42";
    let Ok(number) = text.parse::<i32>() else {
        println!("Could not parse");
        return;
    };
    println!("Parsed: {number}");
}
"#,
                ),
            ],
        ),
        // ── Chapter 7: Packages, Crates, Modules ─────────────────────────────
        (
            "ch07_modules",
            "",
            vec![
                (
                    "inline_modules",
                    r#"#![allow(dead_code)]
// Ch 7 – Modules (inline style)
// Modules organise code into namespaces and control visibility.
// In a real project each mod lives in its own file — see cli_guide.rs.

mod front_of_house {
    pub mod hosting {
        pub fn add_to_waitlist() { println!("Added to waitlist!"); }
        pub fn seat_at_table()   { println!("Seated at table."); }
    }
    pub mod serving {
        pub fn take_order()  { println!("Order taken."); }
        pub fn serve_order() { println!("Order served."); }
    }
}

mod back_of_house {
    pub struct Breakfast {
        pub toast: String,
        seasonal_fruit: String, // private — only constructable via associated fn
    }
    impl Breakfast {
        pub fn summer(toast: &str) -> Self {
            Self { toast: toast.to_string(), seasonal_fruit: String::from("peaches") }
        }
        pub fn describe(&self) { println!("Toast: {}, fruit: {}", self.toast, self.seasonal_fruit); }
    }
    pub enum Appetizer { Soup, Salad }
}

// 'use' brings a path into scope
use crate::front_of_house::hosting;

fn main() {
    // Absolute path
    crate::front_of_house::hosting::add_to_waitlist();
    // via use
    hosting::seat_at_table();
    front_of_house::serving::take_order();
    front_of_house::serving::serve_order();

    // Struct with private field — can only be constructed via Breakfast::summer
    let mut meal = back_of_house::Breakfast::summer("Rye");
    meal.toast = String::from("Wheat"); // public field — OK
    // meal.seasonal_fruit = ...; // ← compile error: private
    meal.describe();

    // Enum variants are all public when the enum is pub
    let order = back_of_house::Appetizer::Soup;
    match order {
        back_of_house::Appetizer::Soup  => println!("Soup"),
        back_of_house::Appetizer::Salad => println!("Salad"),
    }
}
"#,
                ),
                (
                    "paths_and_use",
                    r#"// Ch 7 – Paths, use, as, pub use, glob

use std::collections::{HashMap, HashSet}; // nested path — import both at once
use std::fmt::Result as FmtResult;        // 'as' alias avoids name clash

// pub use re-exports — callers can refer to shapes::Circle directly
mod geometry {
    pub use self::primitives::Circle; // re-export from inner module
    pub mod primitives {
        pub struct Circle { pub radius: f64 }
        impl Circle {
            pub fn area(&self) -> f64 { std::f64::consts::PI * self.radius * self.radius }
            pub fn circumference(&self) -> f64 { 2.0 * std::f64::consts::PI * self.radius }
        }
    }
}
use geometry::Circle; // works thanks to pub use re-export

fn _demo_result() -> FmtResult { Ok(()) }

fn main() {
    let mut scores: HashMap<&str, i32> = HashMap::new();
    scores.insert("Alice", 10);
    scores.insert("Bob", 20);
    let mut pairs: Vec<_> = scores.iter().collect();
    pairs.sort_by_key(|&(k, _)| *k);
    for (k, v) in pairs { println!("{k}: {v}"); }

    let mut unique: HashSet<i32> = HashSet::from([1, 2, 3, 2, 1]);
    unique.insert(4);
    let mut sorted: Vec<_> = unique.iter().collect();
    sorted.sort();
    println!("unique sorted: {:?}", sorted);

    let c = Circle { radius: 5.0 };
    println!("Circle area={:.2} circumference={:.2}", c.area(), c.circumference());
    println!("\nFor multi-file modules → run cli_guide.rs");
}
"#,
                ),
                (
                    "cli_guide",
                    r#"// Ch 7 – Multi-File Modules: CLI Guide
// ══════════════════════════════════════════════════════════════════
//  Modules in real projects map to the filesystem:
//
//    src/lib.rs  ──────  mod front_of_house;
//    src/front_of_house.rs ──  pub mod hosting;
//    src/front_of_house/
//        hosting.rs ──  pub fn add_to_waitlist() {}
//
//  This file structure CANNOT be created inside a single playground.
//  Jump to your terminal and try it!
//
//  ── STEP BY STEP ───────────────────────────────────────────────
//
//  $ cargo new restaurant --lib
//  $ cd restaurant
//
//  Edit src/lib.rs:
//    pub mod front_of_house;
//    pub fn eat_at_restaurant() {
//        front_of_house::hosting::add_to_waitlist();
//    }
//
//  Create src/front_of_house.rs:
//    pub mod hosting;
//
//  Create src/front_of_house/hosting.rs:
//    pub fn add_to_waitlist() { println!("added!"); }
//
//  $ cargo build
//
//  ── READ MORE ──────────────────────────────────────────────────
//  https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html
// ══════════════════════════════════════════════════════════════════

fn main() {
    println!("Ch 7 – Multi-file modules need the terminal.");
    println!();
    println!("  cargo new restaurant --lib");
    println!("  cd restaurant");
    println!("  # then follow the steps in the comments above ↑");
    println!();
    println!("Full chapter: https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html");
}
"#,
                ),
            ],
        ),
        // ── Chapter 8: Collections ────────────────────────────────────────────
        (
            "ch08_collections",
            "",
            vec![
                (
                    "vectors",
                    r#"#![allow(dead_code)]
// Ch 8 – Vectors
// Vec<T>: growable, heap-allocated, same-type sequence.

fn main() {
    let v1: Vec<i32> = Vec::new();
    let v2 = vec![1, 2, 3, 4, 5];
    println!("v1 is empty: {}", v1.is_empty());
    println!("v2: {:?}", v2);

    // Building a vector
    let mut v: Vec<i32> = Vec::new();
    for i in 0..5 { v.push(i * i); }
    println!("squares: {:?}", v);

    // Index access (panics if out-of-bounds)
    println!("v[2] = {}", v[2]);

    // Safe access via .get() — returns Option
    println!("v.get(10) = {:?}", v.get(10)); // None, not a panic

    // Iterating
    for val in &v { print!("{val} "); }
    println!();

    // Mutating while iterating
    let mut v2 = vec![1, 2, 3, 4, 5];
    for val in &mut v2 { *val += 10; }
    println!("after +10: {:?}", v2);

    // Enum trick — store mixed types in a Vec
    #[derive(Debug)]
    enum Cell { Int(i32), Float(f64), Text(String) }
    let row = vec![
        Cell::Int(3),
        Cell::Float(10.12),
        Cell::Text(String::from("hello")),
    ];
    println!("mixed row: {:?}", row);
}
"#,
                ),
                (
                    "strings",
                    r#"// Ch 8 – Strings
// Two string types: &str (borrowed slice) and String (owned, growable).

fn main() {
    // Creating Strings
    let mut s = String::new();
    s.push_str("hello");
    s.push(' ');
    s.push_str("world");
    println!("{s}");

    let s2 = "initial content".to_string();
    let s3 = String::from("from_literal");
    println!("{s2}, {s3}");

    // Concatenation with + (takes ownership of s1)
    let s1 = String::from("Hello, ");
    let s2 = String::from("world!");
    let s3 = s1 + &s2;  // s1 is moved; s2 is borrowed
    println!("{s3}");

    // format! — doesn't take ownership of any argument
    let a = String::from("tic");
    let b = String::from("tac");
    let c = String::from("toe");
    println!("{}", format!("{a}-{b}-{c}"));

    // String length is in *bytes*, not characters
    let hello = "Здравствуйте"; // Russian
    println!("'{hello}' byte length = {}", hello.len());          // 24
    println!("            char count = {}", hello.chars().count()); // 12

    // Iterating characters
    for ch in "नमस्ते".chars() { print!("{ch} "); }
    println!();

    // Slicing (must land on char boundaries — otherwise runtime panic)
    let s = String::from("hello world");
    println!("first 5 bytes: {}", &s[..5]);
}
"#,
                ),
                (
                    "hash_maps",
                    r#"// Ch 8 – Hash Maps
// HashMap<K, V>: key-value store, O(1) average lookup, unordered.

use std::collections::HashMap;

fn main() {
    let mut scores: HashMap<String, i32> = HashMap::new();
    scores.insert(String::from("Alice"), 10);
    scores.insert(String::from("Bob"),   20);

    // Build from two iterators with zip + collect
    let teams  = ["Red", "Blue", "Green"];
    let points = [50i32, 60, 70];
    let scores2: HashMap<_, _> = teams.iter().zip(points.iter()).collect();
    println!("{scores2:?}");

    // Accessing — .get() returns Option<&V>
    if let Some(s) = scores.get("Alice") { println!("Alice → {s}"); }

    // Iterating (order not guaranteed)
    let mut pairs: Vec<_> = scores.iter().collect();
    pairs.sort_by_key(|&(k, _)| k.as_str());
    for (k, v) in &pairs { println!("  {k}: {v}"); }

    // Overwrite an existing value
    scores.insert(String::from("Alice"), 25);

    // Insert only if key is absent — the "entry" API
    scores.entry(String::from("Charlie")).or_insert(50);
    scores.entry(String::from("Alice")).or_insert(999); // won't change Alice
    println!("{scores:?}");

    // Update a value based on the old one — classic word count
    let text = "hello world wonderful world hello rust world";
    let mut word_count: HashMap<&str, u32> = HashMap::new();
    for word in text.split_whitespace() {
        let count = word_count.entry(word).or_insert(0);
        *count += 1;
    }
    let mut wc: Vec<_> = word_count.iter().collect();
    wc.sort_by_key(|&(w, _)| *w);
    for (word, count) in wc { println!("  '{word}': {count}"); }
}
"#,
                ),
            ],
        ),
        // ── Chapter 9: Error Handling ─────────────────────────────────────────
        (
            "ch09_errors",
            "",
            vec![
                (
                    "panicking",
                    r#"// Ch 9 – Unrecoverable Errors with panic!
// Use panic! when the program reaches a state it can't recover from.
// In practice, prefer Result for most errors.

fn divide(a: i32, b: i32) -> i32 {
    if b == 0 { panic!("divide by zero! a={a}"); }
    a / b
}

fn main() {
    // Index out of bounds — would panic:
    let v = vec![1, 2, 3];
    // let _ = v[99]; // ← uncomment to see: index out of bounds

    // Safe alternative
    println!("v.get(1)  = {:?}", v.get(1));
    println!("v.get(99) = {:?}", v.get(99)); // None, not a panic

    // Custom panic
    println!("10 / 2 = {}", divide(10, 2));
    // divide(1, 0); // ← uncomment to trigger panic

    // unwrap() panics on None/Err; expect() adds a message
    let s: Option<&str> = Some("hello");
    println!("unwrap: {}", s.unwrap());

    // When to panic:
    //  • Tests (panic is the test failure mechanism)
    //  • Examples / prototypes
    //  • Contract violations — bugs that "can't" happen
    //  • After checking a condition that proves safety (unsafe code)
    println!("Program completed — no panic triggered.");
}
"#,
                ),
                (
                    "result_type",
                    r#"// Ch 9 – Recoverable Errors with Result<T, E>
// Result encodes success (Ok) or failure (Err) in the type system.

use std::num::ParseIntError;

fn parse_and_double(s: &str) -> Result<i32, ParseIntError> {
    let n: i32 = s.trim().parse()?; // ? propagates error on Err
    Ok(n * 2)
}

fn main() {
    // Basic match on Result
    let good: Result<i32, _> = "42".parse();
    let bad:  Result<i32, _> = "hello".parse::<i32>();
    match good { Ok(n) => println!("Parsed: {n}"),  Err(e) => println!("Error: {e}") }
    match bad  { Ok(n) => println!("Parsed: {n}"),  Err(e) => println!("Error: {e}") }

    // Combinators
    let doubled = "5".parse::<i32>().map(|n| n * 2).unwrap_or(0);
    println!("doubled: {doubled}");

    let n = "bad".parse::<i32>().unwrap_or_else(|e| {
        println!("parse failed ({e}), using -1");
        -1
    });
    println!("n = {n}");

    // Predicates
    println!("\"5\" ok? {}", "5".parse::<i32>().is_ok());
    println!("\"x\" ok? {}", "x".parse::<i32>().is_ok());

    // ? operator in a helper function
    println!("{:?}", parse_and_double("  21 "));
    println!("{:?}", parse_and_double("abc"));

    // and_then chains Results
    let result = "3".parse::<i32>()
        .and_then(|n| if n > 0 { Ok(n * 100) } else { Err("zero".parse::<i32>().unwrap_err()) });
    println!("and_then: {:?}", result);
}
"#,
                ),
                (
                    "propagating",
                    r#"// Ch 9 – Propagating Errors with ? and Custom Error Types
// ? unwraps Ok or returns Err early. main() can return Result too.

use std::fmt;
use std::num::ParseIntError;

#[derive(Debug)]
enum AppError {
    Parse(ParseIntError),
    OutOfRange(i32),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Parse(e)      => write!(f, "parse error: {e}"),
            AppError::OutOfRange(n) => write!(f, "{n} is out of 1–100"),
        }
    }
}

// From<ParseIntError> lets ? convert automatically
impl From<ParseIntError> for AppError {
    fn from(e: ParseIntError) -> Self { AppError::Parse(e) }
}

fn parse_percentage(s: &str) -> Result<i32, AppError> {
    let n: i32 = s.trim().parse()?; // ? converts ParseIntError → AppError
    if n < 1 || n > 100 { return Err(AppError::OutOfRange(n)); }
    Ok(n)
}

// main can return Result — Box<dyn Error> accepts any error type
fn main() -> Result<(), Box<dyn std::error::Error>> {
    for input in ["50", "0", "bad", "75", "200"] {
        match parse_percentage(input) {
            Ok(pct) => println!("'{input}' → {pct}%"),
            Err(e)  => println!("'{input}' → Error: {e}"),
        }
    }
    Ok(())
}
"#,
                ),
            ],
        ),
        // ── Chapter 10: Generics, Traits, Lifetimes ───────────────────────────
        (
            "ch10_generics",
            "",
            vec![
                (
                    "generics",
                    r#"#![allow(dead_code)]
// Ch 10 – Generic Types
// Generics write code once that works across multiple types.

// Generic function — T must implement PartialOrd
fn largest<T: PartialOrd>(list: &[T]) -> &T {
    let mut best = &list[0];
    for item in list { if item > best { best = item; } }
    best
}

// Generic struct
#[derive(Debug)]
struct Pair<T> { first: T, second: T }

impl<T> Pair<T> {
    fn new(first: T, second: T) -> Self { Self { first, second } }
}

// Conditional impl — only when T: Display + PartialOrd
impl<T: std::fmt::Display + PartialOrd> Pair<T> {
    fn cmp_display(&self) {
        if self.first >= self.second { println!("First:  {}", self.first); }
        else                         { println!("Second: {}", self.second); }
    }
}

// Generic enum (like std's Option and Result)
#[derive(Debug)]
enum MyOption<T> { Something(T), Nothing }

fn main() {
    println!("largest int:  {}", largest(&[34, 50, 25, 100, 65]));
    println!("largest char: {}", largest(&['y', 'm', 'a', 'q']));

    let pair = Pair::new(5, 10);
    println!("{pair:?}");
    pair.cmp_display();

    let opt: MyOption<i32> = MyOption::Something(42);
    println!("{opt:?}");
}
"#,
                ),
                (
                    "traits",
                    r#"#![allow(dead_code)]
// Ch 10 – Traits
// Traits define shared behaviour — like interfaces in other languages.

trait Summary {
    fn summarise_author(&self) -> String;

    // Default method — implementors may override it
    fn summarise(&self) -> String {
        format!("(Read more from {}…)", self.summarise_author())
    }
}

struct NewsArticle { headline: String, author: String }
struct Tweet       { username: String, content: String }

impl Summary for NewsArticle {
    fn summarise_author(&self) -> String { self.author.clone() }
    fn summarise(&self) -> String {
        format!("{}, by {}", self.headline, self.author)
    }
}

impl Summary for Tweet {
    fn summarise_author(&self) -> String { format!("@{}", self.username) }
    // Uses the default summarise()
}

// impl Trait syntax — accept any type implementing Summary
fn notify(item: &impl Summary) {
    println!("Breaking: {}", item.summarise());
}

// Trait bound syntax — equivalent, cleaner for multiple params
fn notify_both<T: Summary>(a: &T, b: &T) {
    println!("1: {}", a.summarise());
    println!("2: {}", b.summarise());
}

// Returning impl Trait — concrete type is hidden
fn make_tweet() -> impl Summary {
    Tweet { username: String::from("rustlang"), content: String::from("exciting news!") }
}

fn main() {
    let article = NewsArticle { headline: String::from("Big News"), author: String::from("Jane") };
    let tweet   = Tweet       { username: String::from("horse_ebooks"), content: String::from("of course") };

    notify(&article);
    notify(&tweet);
    notify_both(&tweet, &tweet);

    let t = make_tweet();
    println!("auto: {}", t.summarise());
}
"#,
                ),
                (
                    "lifetimes",
                    r#"// Ch 10 – Lifetime Annotations
// Lifetimes tell the compiler how long references remain valid.
// They prevent dangling references — checked entirely at compile time.

// 'a: returned reference lives as long as the shorter of x and y
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}

// Struct holding a reference needs a lifetime annotation
struct Important<'a> { part: &'a str }

impl<'a> Important<'a> {
    fn level(&self) -> i32 { 3 }

    fn announce_and_return(&self, ann: &str) -> &str {
        println!("Attention (level {}): {ann}", self.level());
        self.part
    }
}

// Combining generics + trait bounds + lifetimes
use std::fmt::Display;
fn longest_with_announcement<'a, T>(x: &'a str, y: &'a str, ann: T) -> &'a str
where T: Display {
    println!("Announcement: {ann}");
    if x.len() > y.len() { x } else { y }
}

fn main() {
    let s1 = String::from("long string is long");
    let result;
    {
        let s2 = String::from("xyz");
        result = longest(s1.as_str(), s2.as_str());
        println!("longest: {result}"); // result used inside s2's lifetime — OK
    }

    let novel  = String::from("Call me Ishmael. Some years ago...");
    let first  = novel.split('.').next().expect("no '.'");
    let imp    = Important { part: first };
    println!("Important: '{}'", imp.announce_and_return("note"));

    let l = longest_with_announcement("hello!", "hi", "side note");
    println!("longest: {l}");

    // 'static — reference valid for the whole program (string literals)
    let s: &'static str = "I live forever";
    println!("{s}");
}
"#,
                ),
            ],
        ),
        // ── Chapter 11: Writing Automated Tests ──────────────────────────────
        (
            "ch11_testing",
            "",
            vec![
                (
                    "writing_tests",
                    r#"// Ch 11 – Writing Tests
// Tests are #[test]-annotated functions compiled (and run) by `cargo test`.
// `cargo run` only executes main() — tests are ignored.
//
// TO RUN THE TESTS IN THIS FILE:
//   Open Terminal, then:
//   cd ~/Library/Application\ Support/com.rustic-playground.app/projects/ch11_testing
//   cargo test writing_tests      ← run tests in this binary
//   cargo test -- --nocapture     ← see println! inside tests
//
// The main() below manually exercises the same logic so ⌘R shows results.

fn add(a: i32, b: i32) -> i32 { a + b }

fn greeting(name: &str) -> String { format!("Hello, {name}!") }

#[derive(Debug)]
struct Rectangle { width: f64, height: f64 }
impl Rectangle {
    fn can_hold(&self, other: &Rectangle) -> bool {
        self.width > other.width && self.height > other.height
    }
}

#[cfg(test)] // compiled only during `cargo test`
mod tests {
    use super::*;

    #[test]
    fn test_add() { assert_eq!(add(2, 3), 5); }

    #[test]
    fn greeting_contains_name() {
        let g = greeting("Carol");
        assert!(g.contains("Carol"), "greeting was '{g}'");
    }

    #[test]
    fn larger_can_hold_smaller() {
        let big   = Rectangle { width: 8.0, height: 7.0 };
        let small = Rectangle { width: 5.0, height: 1.0 };
        assert!(big.can_hold(&small));
    }

    #[test]
    fn smaller_cannot_hold_larger() {
        let big   = Rectangle { width: 8.0, height: 7.0 };
        let small = Rectangle { width: 5.0, height: 1.0 };
        assert!(!small.can_hold(&big));
    }

    #[test]
    #[should_panic(expected = "oh no")]
    fn it_panics() { panic!("oh no, disaster"); }

    #[test]
    fn returns_result() -> Result<(), String> {
        if add(2, 2) == 4 { Ok(()) } else { Err(String::from("2+2 ≠ 4")) }
    }

    #[test]
    #[ignore] // run only with: cargo test -- --ignored
    fn slow_test() { assert!(true); }
}

fn main() {
    println!("=== Manual demo (cargo run) ===");
    println!("add(2,3) = {}", add(2, 3));
    println!("{}", greeting("Rustacean"));
    let big   = Rectangle { width: 8.0, height: 7.0 };
    let small = Rectangle { width: 5.0, height: 1.0 };
    println!("big holds small? {}", big.can_hold(&small));
    println!();
    println!("To run the #[test] functions:");
    println!("  cd ~/Library/Application\\ Support/com.rustic-playground.app/projects/ch11_testing");
    println!("  cargo test writing_tests");
}
"#,
                ),
                (
                    "test_organisation",
                    r#"// Ch 11 – Test Organisation
// Unit tests sit next to the code they test (in the same file).
// Integration tests live in a top-level tests/ directory.
// Only libraries (lib.rs) can have integration tests — binaries cannot.
//
// RUN THESE TESTS:
//   cd ~/Library/Application\ Support/com.rustic-playground.app/projects/ch11_testing
//   cargo test test_organisation   ← unit tests in this binary
//   cargo test                     ← all tests in the project

pub fn divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 { Err(String::from("division by zero")) }
    else        { Ok(a / b) }
}

pub fn is_palindrome(s: &str) -> bool {
    let chars: Vec<char> = s.chars().collect();
    chars == chars.iter().rev().cloned().collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {
    use super::*;

    // assert_eq! / assert_ne!
    #[test]
    fn divide_works()    { assert_eq!(divide(10.0, 2.0), Ok(5.0)); }
    #[test]
    fn divide_by_zero()  { assert!(divide(1.0, 0.0).is_err()); }

    // Custom failure messages
    #[test]
    fn palindrome_racecar() {
        assert!(is_palindrome("racecar"), "'racecar' should be a palindrome");
    }
    #[test]
    fn not_palindrome_hello() {
        assert!(!is_palindrome("hello"), "'hello' should NOT be a palindrome");
    }

    // assert_ne!
    #[test]
    fn divide_five_by_two() {
        assert_ne!(divide(5.0, 2.0), Ok(3.0));
    }
}

fn main() {
    println!("divide(10,2) = {:?}", divide(10.0, 2.0));
    println!("divide(1,0)  = {:?}", divide(1.0, 0.0));
    println!("is_palindrome(\"racecar\") = {}", is_palindrome("racecar"));
    println!("is_palindrome(\"hello\")   = {}", is_palindrome("hello"));
    println!();
    println!("Run `cargo test test_organisation` to execute the #[test] functions.");
    println!();
    println!("Integration tests (tests/ directory) only work with lib crates:");
    println!("  cargo new mylib --lib");
    println!("  mkdir tests && echo 'use mylib::...' > tests/integration.rs");
    println!("  cargo test");
}
"#,
                ),
            ],
        ),
        // ── Chapter 12: An I/O Project ────────────────────────────────────────
        (
            "ch12_minigrep",
            "",
            vec![
                (
                    "minigrep",
                    r#"// Ch 12 – minigrep (playground edition)
// Searches for a pattern in a file from the content folder.
//
// HOW TO USE:
//   1. Add a text file (e.g. poem.txt) via the Files panel of this project.
//   2. Change SEARCH_TERM and FILENAME below, then press ⌘R.
//
// The book version reads query + filename from std::env::args() — see cli_guide.rs.

use std::{env, fs};

const SEARCH_TERM: &str = "the";
const FILENAME:    &str = "poem.txt";

fn search<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    contents.lines().filter(|l| l.contains(query)).collect()
}

fn search_insensitive<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    let q = query.to_lowercase();
    contents.lines().filter(|l| l.to_lowercase().contains(&q)).collect()
}

fn main() {
    let content_dir = env::var("PLAYGROUND_CONTENT").unwrap_or_default();
    let path = format!("{content_dir}/{FILENAME}");

    let contents = fs::read_to_string(&path).unwrap_or_else(|_| {
        eprintln!("'{FILENAME}' not found — using built-in poem. Add your own via Files panel.");
        String::from("\
I'm nobody! Who are you?
Are you nobody, too?
Then there's a pair of us — don't tell!
They'd banish us, you know.
How dreary to be somebody!
How public, like a frog
To tell your name the livelong day
To an admiring bog!")
    });

    println!("=== Case-sensitive: '{}' ===", SEARCH_TERM);
    let hits = search(SEARCH_TERM, &contents);
    if hits.is_empty() { println!("(no matches)"); }
    else { for line in hits { println!("  {line}"); } }

    println!("\n=== Case-insensitive: '{}' ===", SEARCH_TERM);
    for line in search_insensitive(SEARCH_TERM, &contents) { println!("  {line}"); }
}
"#,
                ),
                (
                    "cli_guide",
                    r#"// Ch 12 – minigrep: CLI Edition
// ══════════════════════════════════════════════════════════════════
//  The book builds minigrep as a real command-line tool that reads
//  its arguments from the shell: `cargo run -- searchterm file.txt`
//
//  This playground can't pass custom CLI args — use the terminal.
//
//  ── BUILD THE REAL VERSION ─────────────────────────────────────
//
//  $ cargo new minigrep
//  $ cd minigrep
//  # Paste the code from https://doc.rust-lang.org/book/ch12-01-accepting-command-line-arguments.html
//  # into src/main.rs, then:
//
//  $ echo "I'm nobody! Who are you?" > poem.txt
//  $ cargo run -- the poem.txt
//  $ CASE_INSENSITIVE=1 cargo run -- TO poem.txt
//
//  ── WHAT THE CHAPTER COVERS ────────────────────────────────────
//  • std::env::args() — reading command-line arguments
//  • std::fs::read_to_string() — reading files
//  • Separating logic into a Config struct and a run() function
//  • Writing errors to stderr with eprintln!
//  • Test-driven development (writing tests first)
//  • Environment variables with std::env::var()
//
//  ── READ MORE ──────────────────────────────────────────────────
//  https://doc.rust-lang.org/book/ch12-00-an-io-project.html
// ══════════════════════════════════════════════════════════════════

use std::env;

fn main() {
    // Show what args look like (this playground has no custom args)
    let args: Vec<String> = env::args().collect();
    println!("This binary was invoked as: {}", args[0]);
    println!("Extra args passed: {}", args.len().saturating_sub(1));
    println!();
    println!("In the real minigrep you would run:");
    println!("  cargo run -- <query> <filename>");
    println!("  CASE_INSENSITIVE=1 cargo run -- <query> <filename>");
    println!();
    println!("Full chapter: https://doc.rust-lang.org/book/ch12-00-an-io-project.html");
}
"#,
                ),
            ],
        ),
        // ── Chapter 13: Closures and Iterators ────────────────────────────────
        (
            "ch13_closures",
            "",
            vec![
                (
                    "closures",
                    r#"// Ch 13 – Closures
// Closures are anonymous functions that capture their enclosing environment.

fn apply<F: Fn(i32) -> i32>(f: F, x: i32) -> i32 { f(x) }
fn apply_twice<F: Fn(i32) -> i32>(f: F, x: i32) -> i32 { f(f(x)) }

// move: transfer ownership of captured variables into the closure
fn make_adder(n: i32) -> impl Fn(i32) -> i32 { move |x| x + n }

fn main() {
    // Closure syntax: |params| expression
    let square  = |x: i32| x * x;
    let add_one = |x| x + 1;        // types are inferred
    let greet   = |name: &str| println!("Hello, {name}!");

    println!("{}", square(5));
    println!("{}", add_one(10));
    greet("Rustacean");

    // Capturing environment by reference
    let threshold = 10;
    let is_big = |&n: &i32| n > threshold;
    let numbers = vec![5, 12, 3, 18, 7, 25];
    let big: Vec<_> = numbers.iter().filter(|n| is_big(n)).collect();
    println!(">{threshold}: {:?}", big);

    // Higher-order functions
    println!("apply square(4)       = {}", apply(square, 4));
    println!("apply_twice add_one 0 = {}", apply_twice(add_one, 0));

    // Returned closures capture by move
    let add5  = make_adder(5);
    let add10 = make_adder(10);
    println!("add5(3)={}, add10(3)={}", add5(3), add10(3));

    // FnMut — closure that mutates its captured state
    let mut counter = 0;
    let mut inc = || { counter += 1; counter };
    println!("{} {} {}", inc(), inc(), inc());
}
"#,
                ),
                (
                    "iterators",
                    r#"// Ch 13 – Iterators
// Iterators process a sequence lazily — nothing runs until consumed.

fn main() {
    let v: Vec<i32> = (1..=10).collect();

    // ── Adapters (lazy — transform the iterator) ──────────────────────────────

    // map — transform each element
    let doubled: Vec<_> = v.iter().map(|x| x * 2).collect();
    println!("doubled: {:?}", doubled);

    // filter — keep elements where predicate is true
    let evens: Vec<_> = v.iter().filter(|&&x| x % 2 == 0).collect();
    println!("evens: {:?}", evens);

    // Chaining adapters — lazy until .collect()
    let odd_squares: Vec<i32> = v.iter()
        .filter(|&&x| x % 2 != 0)
        .map(|&x| x * x)
        .collect();
    println!("odd squares: {:?}", odd_squares);

    // ── Consumers (eager — produce a final value) ─────────────────────────────

    let sum: i32 = v.iter().sum();
    let product: i32 = v.iter().product();
    println!("sum={sum}, product={product}");

    // fold — general-purpose reduction
    let sum2 = v.iter().fold(0, |acc, &x| acc + x);
    println!("fold sum = {sum2}");

    // count, max, min
    println!("count={} max={:?} min={:?}", v.len(), v.iter().max(), v.iter().min());

    // any / all
    println!("any > 9? {}", v.iter().any(|&x| x > 9));
    println!("all > 0? {}", v.iter().all(|&x| x > 0));

    // position / find
    println!("first even pos: {:?}", v.iter().position(|&x| x % 2 == 0));
    println!("first > 7:      {:?}", v.iter().find(|&&x| x > 7));
}
"#,
                ),
                (
                    "combinators",
                    r#"// Ch 13 – Iterator Combinators
// Composing iterators for expressive, zero-overhead data pipelines.

fn main() {
    // enumerate — yields (index, value) pairs
    let words = ["alpha", "beta", "gamma", "delta"];
    for (i, word) in words.iter().enumerate() { println!("{i}: {word}"); }

    // zip — pair two iterators together
    let names  = ["Alice", "Bob", "Carol"];
    let scores = [95u32, 82, 88];
    for (name, score) in names.iter().zip(scores.iter()) {
        println!("{name}: {score}");
    }

    // chain — concatenate two iterators
    let a = [1, 2, 3];
    let b = [4, 5, 6];
    let chained: Vec<_> = a.iter().chain(b.iter()).collect();
    println!("chained: {:?}", chained);

    // flat_map — map then flatten one level
    let sentences = ["hello world", "foo bar baz"];
    let words: Vec<_> = sentences.iter().flat_map(|s| s.split_whitespace()).collect();
    println!("words: {:?}", words);

    // take / skip — slice an iterator
    let first5: Vec<_>   = (1..).take(5).collect();         // infinite range!
    let mid4: Vec<_>     = (1..=10).skip(3).take(4).collect();
    println!("take 5 from 1..: {:?}", first5);
    println!("skip 3 take 4:   {:?}", mid4);

    // step_by
    let every3: Vec<_> = (0..=20).step_by(3).collect();
    println!("every 3rd: {:?}", every3);

    // unzip — split an iterator of pairs into two collections
    let pairs = vec![(1, 'a'), (2, 'b'), (3, 'c')];
    let (nums, chars): (Vec<_>, Vec<_>) = pairs.into_iter().unzip();
    println!("nums={nums:?}  chars={chars:?}");
}
"#,
                ),
            ],
        ),
        // ── Chapter 14: More About Cargo ──────────────────────────────────────
        (
            "ch14_cargo",
            "",
            vec![
                (
                    "cfg_and_build_info",
                    r#"// Ch 14 – Cargo Features and Conditional Compilation
// Feature flags let downstream users opt into optional functionality.
// cfg! and #[cfg(...)] check compile-time conditions.

fn main() {
    // cfg! macro — evaluates to bool at compile time
    let mode = if cfg!(debug_assertions) { "DEBUG" } else { "RELEASE" };
    println!("Build mode: {mode}");

    // Target OS and architecture
    let os = if      cfg!(target_os = "macos")   { "macOS" }
              else if cfg!(target_os = "linux")   { "Linux" }
              else if cfg!(target_os = "windows") { "Windows" }
              else                                 { "other" };
    let arch = if      cfg!(target_arch = "x86_64")  { "x86_64" }
                else if cfg!(target_arch = "aarch64") { "aarch64 (Apple Silicon)" }
                else                                   { "other" };
    println!("Platform: {os} / {arch}");

    // Build metadata injected by Cargo via env! macro
    println!("Package:  {} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    println!("Edition:  Rust 2021");

    // #[cfg(...)] conditionally compiles a block
    #[cfg(debug_assertions)]
    println!("(debug-only) This line won't appear in --release builds.");

    println!();
    println!("See cli_guide.rs for workspace, profiles, and publishing info.");
}
"#,
                ),
                (
                    "cli_guide",
                    r#"// Ch 14 – More About Cargo and Crates.io: CLI Guide
// ══════════════════════════════════════════════════════════════════
//  Chapter 14 is about the Cargo *tool* — profiles, workspaces,
//  documentation, and publishing to crates.io.  These concepts
//  can't be demonstrated inside a playground; try them in a terminal.
//
//  ── RELEASE PROFILES ───────────────────────────────────────────
//  # Cargo.toml
//  [profile.dev]
//  opt-level = 0          # fast compile, slow binary
//
//  [profile.release]
//  opt-level = 3          # slow compile, fast binary
//
//  $ cargo build            # dev profile
//  $ cargo build --release  # release profile
//
//  ── DOCUMENTATION ──────────────────────────────────────────────
//  /// Doc comments use Markdown
//  /// # Examples
//  /// ```
//  /// assert_eq!(my_crate::add(1,1), 2);
//  /// ```
//  pub fn add(a: i32, b: i32) -> i32 { a + b }
//
//  $ cargo doc --open       # build + open docs in browser
//
//  ── WORKSPACES ─────────────────────────────────────────────────
//  A workspace shares one Cargo.lock across multiple crates:
//
//  my_workspace/
//  ├── Cargo.toml    [workspace] members = ["adder", "add_one"]
//  ├── adder/
//  └── add_one/
//
//  $ cargo build             # builds all workspace members
//  $ cargo test -p add_one   # test one member
//
//  ── PUBLISHING TO CRATES.IO ────────────────────────────────────
//  $ cargo login             # authenticate with your API token
//  $ cargo publish           # publish the current crate
//
//  ── READ MORE ──────────────────────────────────────────────────
//  https://doc.rust-lang.org/book/ch14-00-more-about-cargo.html
// ══════════════════════════════════════════════════════════════════

fn main() {
    println!("Ch 14 is about Cargo the tool — try these commands in a terminal:");
    println!();
    println!("  cargo build --release     # optimised build");
    println!("  cargo doc --open          # build and view docs");
    println!("  cargo test                # run all tests");
    println!("  cargo publish             # publish to crates.io");
    println!();
    println!("Full chapter: https://doc.rust-lang.org/book/ch14-00-more-about-cargo.html");
}
"#,
                ),
            ],
        ),
        // ── Chapter 15: Smart Pointers ────────────────────────────────────────
        (
            "ch15_smart_pointers",
            "",
            vec![
                (
                    "box_t",
                    r#"#![allow(dead_code)]
// Ch 15 – Box<T>
// Box stores a value on the heap. Use when:
//  • You have a large value and don't want to copy it
//  • You need a recursive type (unknown size at compile time)
//  • You need a trait object (Box<dyn Trait>)

// Recursive type — without Box the size would be infinite
#[derive(Debug)]
enum List { Cons(i32, Box<List>), Nil }
use List::{Cons, Nil};

// Trait object — any type implementing Draw can be stored
trait Draw { fn draw(&self); }
struct Circle { radius: f64 }
struct Square { side:   f64 }
impl Draw for Circle { fn draw(&self) { println!("Circle  r={:.1}", self.radius); } }
impl Draw for Square { fn draw(&self) { println!("Square  s={:.1}", self.side);   } }

fn main() {
    // Basic heap allocation
    let b = Box::new(5);
    println!("b = {b}"); // deref coercion: Box<i32> acts just like i32

    // Recursive list: 1 → 2 → 3 → Nil
    let list = Cons(1, Box::new(Cons(2, Box::new(Cons(3, Box::new(Nil))))));
    println!("{list:?}");

    // Trait objects — heterogeneous Vec of Drawables
    let shapes: Vec<Box<dyn Draw>> = vec![
        Box::new(Circle { radius: 5.0 }),
        Box::new(Square { side:   3.0 }),
        Box::new(Circle { radius: 1.5 }),
    ];
    for shape in &shapes { shape.draw(); }

    // Deref coercion: &Box<String> → &String → &str automatically
    let x = Box::new(String::from("hello"));
    println!("length via deref coercion: {}", x.len());
}
"#,
                ),
                (
                    "rc_t",
                    r#"// Ch 15 – Rc<T>: Reference Counting
// Rc<T> allows multiple owners of the same heap value (single-threaded only).
// The value is freed when the last Rc is dropped.

use std::rc::{Rc, Weak};
use std::cell::RefCell;

fn main() {
    // Multiple owners via Rc::clone (cheap — only increments count)
    let a = Rc::new(String::from("shared"));
    let b = Rc::clone(&a);
    let c = Rc::clone(&a);
    println!("a={a}, b={b}, c={c}");
    println!("ref count = {}", Rc::strong_count(&a)); // 3

    drop(c);
    println!("after drop(c): count = {}", Rc::strong_count(&a)); // 2
    // a and b still valid

    // Rc<RefCell<T>> — shared ownership + interior mutability
    let shared = Rc::new(RefCell::new(vec![1, 2, 3]));
    let clone1 = Rc::clone(&shared);
    let clone2 = Rc::clone(&shared);

    clone1.borrow_mut().push(4);
    clone2.borrow_mut().push(5);
    println!("shared vec: {:?}", shared.borrow());

    // Weak<T> — non-owning reference that doesn't prevent drop
    // Useful for parent→child back-pointers (avoids reference cycles)
    let strong = Rc::new(42);
    let weak: Weak<i32> = Rc::downgrade(&strong);
    println!("strong={}, weak upgrades to {:?}", Rc::strong_count(&strong), weak.upgrade());

    drop(strong); // ref count → 0, value freed
    println!("after drop: weak upgrades to {:?}", weak.upgrade()); // None
}
"#,
                ),
                (
                    "refcell",
                    r#"// Ch 15 – RefCell<T> and Interior Mutability
// RefCell enforces borrowing rules at *runtime* instead of compile time.
// Useful when you need mutation via an immutable reference.

use std::cell::RefCell;

// Example: a mock that records sent messages
// send() takes &self but needs to mutate sent_messages — classic use case
trait Messenger { fn send(&self, msg: &str); }

struct MockMessenger { sent_messages: RefCell<Vec<String>> }
impl MockMessenger {
    fn new() -> Self { Self { sent_messages: RefCell::new(vec![]) } }
}
impl Messenger for MockMessenger {
    fn send(&self, msg: &str) {
        self.sent_messages.borrow_mut().push(msg.to_string());
    }
}

fn main() {
    // RefCell basics
    let data = RefCell::new(vec![1, 2, 3]);
    data.borrow_mut().push(4);
    println!("{:?}", data.borrow());

    // Multiple immutable borrows at once — OK
    {
        let r1 = data.borrow();
        let r2 = data.borrow();
        println!("r1={r1:?}, r2={r2:?}");
    } // both released here

    // Mutable borrow — only one at a time (runtime panic if violated)
    { data.borrow_mut().push(5); }
    println!("after push: {:?}", data.borrow());

    // try_borrow_mut — non-panicking
    let x = RefCell::new(10);
    let _held = x.borrow(); // immutable borrow active
    match x.try_borrow_mut() {
        Ok(_)  => println!("got mutable borrow"),
        Err(e) => println!("expected failure: {e}"),
    }

    // Mock example
    let m = MockMessenger::new();
    m.send("Alert: 75% quota");
    m.send("Alert: 90% quota");
    println!("sent: {:?}", m.sent_messages.borrow());
}
"#,
                ),
            ],
        ),
        // ── Chapter 16: Fearless Concurrency ──────────────────────────────────
        (
            "ch16_concurrency",
            "",
            vec![
                (
                    "threads",
                    r#"// Ch 16 – Threads
// Rust's ownership model guarantees thread safety at compile time.
// Data races are a compile error, not a runtime bug.

use std::thread;
use std::time::Duration;

fn main() {
    // Spawn a thread — returns JoinHandle
    let handle = thread::spawn(|| {
        for i in 1..=5 {
            println!("  spawned: {i}");
            thread::sleep(Duration::from_millis(1));
        }
    });

    for i in 1..=3 {
        println!("main: {i}");
        thread::sleep(Duration::from_millis(1));
    }

    handle.join().unwrap(); // wait for the spawned thread to finish

    // move closure — transfer ownership of v into the thread
    let v = vec![1, 2, 3];
    let handle2 = thread::spawn(move || {
        println!("spawned owns v: {:?}", v);
    });
    // println!("{v:?}"); // ← compile error: v was moved
    handle2.join().unwrap();

    // Spawn multiple workers and collect return values
    let handles: Vec<_> = (0..4).map(|i| {
        thread::spawn(move || { println!("  worker {i}"); i * i })
    }).collect();

    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    println!("squares: {:?}", results);
}
"#,
                ),
                (
                    "channels",
                    r#"// Ch 16 – Message Passing with Channels
// mpsc = multiple producer, single consumer.
// "Do not communicate by sharing memory; share memory by communicating."

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn main() {
    // Basic single-producer channel
    let (tx, rx) = mpsc::channel::<String>();
    thread::spawn(move || {
        for msg in ["hello", "from", "the", "thread"] {
            tx.send(msg.to_string()).unwrap();
            thread::sleep(Duration::from_millis(5));
        }
        // tx dropped here — channel closes
    });

    for received in rx {    // iterates until channel closes
        println!("Got: {received}");
    }

    // Multiple producers via Sender::clone
    let (tx2, rx2) = mpsc::channel::<(usize, String)>();
    for id in 0..3 {
        let tx = tx2.clone();
        thread::spawn(move || {
            tx.send((id, format!("worker-{id} reporting in"))).unwrap();
        });
    }
    drop(tx2); // drop original so channel closes when all clones finish

    let mut msgs: Vec<_> = rx2.iter().collect();
    msgs.sort_by_key(|(id, _)| *id);
    for (id, msg) in msgs { println!("[{id}] {msg}"); }

    // try_recv — non-blocking check
    let (tx3, rx3) = mpsc::channel::<i32>();
    tx3.send(42).unwrap();
    println!("try_recv: {:?}", rx3.try_recv()); // Ok(42)
    println!("try_recv: {:?}", rx3.try_recv()); // Err(Empty)
}
"#,
                ),
                (
                    "shared_state",
                    r#"// Ch 16 – Shared-State Concurrency
// Arc<Mutex<T>>: atomic reference counting + mutual exclusion.
// Arc is the thread-safe version of Rc; Mutex provides interior mutability.

use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    // Mutex basics (single-threaded)
    let m = Mutex::new(5);
    {
        let mut num = m.lock().unwrap(); // blocks until lock acquired
        *num = 6;
    } // MutexGuard dropped → lock released automatically
    println!("m = {:?}", m);

    // Arc<Mutex<T>> — shared mutable counter across threads
    let counter = Arc::new(Mutex::new(0u32));
    let handles: Vec<_> = (0..10).map(|_| {
        let c = Arc::clone(&counter);
        thread::spawn(move || { *c.lock().unwrap() += 1; })
    }).collect();
    for h in handles { h.join().unwrap(); }
    println!("counter = {}", *counter.lock().unwrap()); // 10

    // Shared vector — each thread writes to its own slot
    let data = Arc::new(Mutex::new(vec![0u32; 4]));
    let handles: Vec<_> = (0..4).map(|i| {
        let d = Arc::clone(&data);
        thread::spawn(move || { d.lock().unwrap()[i] = (i as u32 + 1) * 10; })
    }).collect();
    for h in handles { h.join().unwrap(); }
    println!("data: {:?}", *data.lock().unwrap()); // [10, 20, 30, 40]
}
"#,
                ),
            ],
        ),
        // ── Chapter 17: OOP Features of Rust ──────────────────────────────────
        (
            "ch17_oop",
            "",
            vec![
                (
                    "encapsulation",
                    r#"// Ch 17 – Encapsulation
// Rust achieves encapsulation through private fields and public methods —
// the same goal as OOP, without inheritance.

pub struct AveragedCollection {
    list:    Vec<f64>,
    average: f64,     // kept in sync automatically — callers can't corrupt it
}

impl AveragedCollection {
    pub fn new() -> Self { Self { list: vec![], average: 0.0 } }

    pub fn add(&mut self, value: f64) {
        self.list.push(value);
        self.update_average();
    }

    pub fn remove(&mut self) -> Option<f64> {
        let result = self.list.pop();
        if result.is_some() { self.update_average(); }
        result
    }

    pub fn average(&self) -> f64 { self.average }
    pub fn len(&self)     -> usize { self.list.len() }
    pub fn is_empty(&self) -> bool { self.list.is_empty() }

    fn update_average(&mut self) { // private — implementation detail
        self.average = if self.list.is_empty() { 0.0 }
                       else { self.list.iter().sum::<f64>() / self.list.len() as f64 };
    }
}

fn main() {
    let mut coll = AveragedCollection::new();
    coll.add(10.0); coll.add(20.0); coll.add(30.0);
    println!("count={} avg={:.1}", coll.len(), coll.average()); // 3, 20.0

    coll.remove();
    println!("after remove: count={} avg={:.1}", coll.len(), coll.average()); // 2, 15.0

    // coll.list    // ← compile error: field `list` is private
    // coll.average // ← compile error: field `average` is private
    // The only way to interact is through the public API — that's encapsulation.
}
"#,
                ),
                (
                    "trait_objects",
                    r#"// Ch 17 – Trait Objects (Dynamic Dispatch)
// Box<dyn Trait> stores values of any type that implements the trait.
// The concrete type is unknown at compile time — dispatch happens at runtime.

trait Draw { fn draw(&self); }

struct Screen { components: Vec<Box<dyn Draw>> }
impl Screen {
    fn render(&self) { for c in &self.components { c.draw(); } }
}

struct Button     { label: String, width: u32, height: u32 }
struct SelectBox  { options: Vec<String> }
struct Slider     { min: f64, max: f64, value: f64 }

impl Draw for Button    { fn draw(&self) { println!("Button   [{}×{}] '{}'", self.width, self.height, self.label); } }
impl Draw for SelectBox { fn draw(&self) { println!("SelectBox options={:?}", self.options); } }
impl Draw for Slider    { fn draw(&self) { println!("Slider   [{:.0}..{:.0}] = {:.1}", self.min, self.max, self.value); } }

// Contrast: generic (monomorphised — static dispatch, zero runtime cost)
fn draw_all_generic<T: Draw>(items: &[T]) { for i in items { i.draw(); } }

fn main() {
    // Heterogeneous collection — each element can be a different concrete type
    let screen = Screen {
        components: vec![
            Box::new(Button    { label: String::from("OK"), width: 50, height: 10 }),
            Box::new(SelectBox { options: vec!["A".into(), "B".into()] }),
            Box::new(Slider    { min: 0.0, max: 100.0, value: 42.5 }),
            Box::new(Button    { label: String::from("Cancel"), width: 50, height: 10 }),
        ],
    };
    screen.render();

    println!();
    // Homogeneous — all the same type; compiler specialises the code
    let buttons = [
        Button { label: String::from("Yes"), width: 40, height: 8 },
        Button { label: String::from("No"),  width: 40, height: 8 },
    ];
    draw_all_generic(&buttons);
}
"#,
                ),
                (
                    "state_pattern",
                    r#"// Ch 17 – State Pattern (type-state idiom)
// The book shows the classic OOP state pattern, then refactors to idiomatic Rust.
// Here we show the type-state version: each state is a *type*, so invalid
// transitions are compile errors rather than runtime panics.

struct Draft         { content: String }
struct PendingReview { content: String }
struct Published     { content: String }

impl Draft {
    fn new() -> Self { Self { content: String::new() } }
    fn add_text(&mut self, text: &str) { self.content.push_str(text); }
    fn request_review(self) -> PendingReview { PendingReview { content: self.content } }
}

impl PendingReview {
    fn approve(self) -> Published { Published { content: self.content } }
    fn reject(self)  -> Draft     { Draft { content: self.content } }
    fn content(&self) -> &str     { "(not visible until published)" }
}

impl Published {
    fn content(&self) -> &str { &self.content }
}

fn main() {
    let mut post = Draft::new();
    post.add_text("I ate a salad for lunch today.");
    post.add_text(" It was delightful.");

    let post = post.request_review();
    // post.add_text("oops"); // ← compile error: no add_text on PendingReview
    println!("During review: '{}'", post.content()); // empty

    // Reject and revise
    let mut post = post.reject();
    post.add_text(" Also had soup.");

    // Approve
    let post = post.request_review().approve();
    println!("Published:     '{}'", post.content());
    // The type system guarantees you can't read unpublished content —
    // no runtime check needed.
}
"#,
                ),
            ],
        ),
        // ── Chapter 18: Patterns and Matching ─────────────────────────────────
        (
            "ch18_patterns",
            "",
            vec![
                (
                    "pattern_syntax",
                    r#"// Ch 18 – Pattern Syntax
// Patterns appear everywhere: match, if let, while let, for, let, fn params.

fn coordinate(point: (i32, i32, i32)) {
    let (x, y, z) = point; // tuple destructuring in let
    println!("x={x} y={y} z={z}");
}

struct Point { x: i32, y: i32 }

enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
    Color(u8, u8, u8),
}

fn describe_message(msg: Message) {
    match msg {
        Message::Quit              => println!("Quit"),
        Message::Move { x, y }    => println!("Move to ({x},{y})"),
        Message::Write(text)       => println!("Write: {text}"),
        Message::Color(r, g, b)   => println!("Color: rgb({r},{g},{b})"),
    }
}

fn main() {
    // Literals, variables, wildcards
    let x = 2;
    match x {
        1             => println!("one"),
        2 | 3         => println!("two or three"),   // | is "or"
        4..=9         => println!("four to nine"),   // ..= inclusive range
        _             => println!("other"),          // _ wildcard
    }

    // Ranges on chars
    let c = 'm';
    match c {
        'a'..='f' => println!("early alphabet"),
        'g'..='z' => println!("late alphabet"),
        _         => println!("non-alpha"),
    }

    coordinate((3, -5, 0));

    // Struct destructuring
    let p = Point { x: 0, y: 7 };
    let Point { x, y } = p;
    println!("point x={x} y={y}");

    // Ignoring values
    let (_, b, _) = (1, 2, 3);
    println!("middle: {b}");

    // .. rest pattern
    let nums = (1, 2, 8, 16, 32);
    let (first, .., last) = nums;
    println!("first={first} last={last}");

    for msg in [
        Message::Move { x: 5, y: 10 },
        Message::Write(String::from("hi")),
        Message::Color(255, 128, 0),
        Message::Quit,
    ] { describe_message(msg); }
}
"#,
                ),
                (
                    "bindings_and_guards",
                    r#"#![allow(dead_code)]
// Ch 18 – Match Guards, @ Bindings, Refutability

#[derive(Debug)]
enum Msg { Hello { id: i32 }, Quit }

fn main() {
    // ── Match guards — extra if condition ─────────────────────────────────────
    let pair = (2, -2);
    match pair {
        (x, y) if x == y        => println!("equal"),
        (x, y) if x + y == 0   => println!("opposites: {x} and {y}"),
        (x, _)                  => println!("x={x}, no special relationship"),
    }

    // Guard with multiple patterns
    let x = 4;
    let y = false;
    match x {
        4 | 5 | 6 if y => println!("yes (y is true)"),
        _               => println!("no (y is false or x out of range)"),
    }

    // ── @ bindings — capture a value while testing it ─────────────────────────
    let msg = Msg::Hello { id: 5 };
    match msg {
        Msg::Hello { id: id @ 3..=7 }  => println!("id in 3-7: {id}"),
        Msg::Hello { id: 10..=12 }      => println!("id in 10-12"),
        Msg::Hello { id }               => println!("other id: {id}"),
        Msg::Quit                        => println!("quit"),
    }

    // ── Refutability ──────────────────────────────────────────────────────────
    // Irrefutable: always matches — required by let / function params
    let (a, b) = (1, 2); // fine: tuples always match
    println!("a={a} b={b}");

    // Refutable: might not match — requires if let / while let
    let opt: Option<i32> = Some(42);
    if let Some(v) = opt { println!("got {v}"); }

    // while let
    let mut stack = vec![1, 2, 3];
    while let Some(top) = stack.pop() { print!("{top} "); }
    println!();

    // for with destructuring
    for (i, val) in ['a', 'b', 'c'].iter().enumerate() { println!("{i}: {val}"); }
}
"#,
                ),
            ],
        ),
        // ── Chapter 19: Advanced Features ─────────────────────────────────────
        (
            "ch19_advanced",
            "",
            vec![
                (
                    "unsafe_rust",
                    r#"#![allow(dead_code)]
// Ch 19 – Unsafe Rust
// unsafe { } unlocks five additional powers:
//   1. Dereference raw pointers
//   2. Call unsafe functions
//   3. Access/modify mutable statics
//   4. Implement unsafe traits
//   5. Access union fields
//
// Safety responsibility shifts to you — the compiler won't check inside.

static HELLO: &str = "Hello from a static!";
static mut COUNTER: u32 = 0;

unsafe fn dangerous_fn() { println!("Inside an unsafe fn."); }

// Safe abstraction wrapping unsafe code
fn split_at_mut(values: &mut [i32], mid: usize) -> (&mut [i32], &mut [i32]) {
    let len = values.len();
    let ptr = values.as_mut_ptr();
    assert!(mid <= len);
    unsafe {
        (
            std::slice::from_raw_parts_mut(ptr, mid),
            std::slice::from_raw_parts_mut(ptr.add(mid), len - mid),
        )
    }
}

fn main() {
    println!("{HELLO}");

    // Raw pointers — can create in safe code, deref only in unsafe
    let mut num = 5i32;
    let r1 = &num     as *const i32;
    let r2 = &mut num as *mut i32;
    unsafe {
        println!("*r1 = {}", *r1);
        *r2 = 42;
        println!("*r2 after write = {}", *r2);
        dangerous_fn();
        COUNTER += 1;
        println!("COUNTER = {}", *std::ptr::addr_of!(COUNTER));
    }

    // Safe abstraction over unsafe internals
    let mut v = vec![1, 2, 3, 4, 5, 6];
    let (left, right) = split_at_mut(&mut v, 3);
    left[0]  = 10;
    right[0] = 40;
    println!("left={left:?} right={right:?}");
}
"#,
                ),
                (
                    "advanced_traits",
                    r#"// Ch 19 – Advanced Traits
// Associated types, operator overloading (default type params), newtype pattern.

use std::fmt;
use std::ops::Add;

// Associated type — Iterator's Item is an associated type, not a generic param.
// This means a type can only implement Iterator once (one Item type).
trait Summarise {
    type Summary; // associated type
    fn summarise(&self) -> Self::Summary;
}

struct Article { title: String, author: String }
impl Summarise for Article {
    type Summary = String;
    fn summarise(&self) -> String { format!("{} by {}", self.title, self.author) }
}

// Operator overloading with default type parameter: Add<Rhs = Self>
#[derive(Debug, Clone, Copy, PartialEq)]
struct Vec2 { x: f64, y: f64 }

impl Add for Vec2 {
    type Output = Vec2;
    fn add(self, rhs: Vec2) -> Vec2 { Vec2 { x: self.x + rhs.x, y: self.y + rhs.y } }
}

// Adding a Vec2 to a (f64, f64) — different Rhs type
impl Add<(f64, f64)> for Vec2 {
    type Output = Vec2;
    fn add(self, (dx, dy): (f64, f64)) -> Vec2 { Vec2 { x: self.x + dx, y: self.y + dy } }
}

// Newtype pattern — wrapper for implementing foreign traits on foreign types
struct Wrapper(Vec<String>);
impl fmt::Display for Wrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]", self.0.join(", "))
    }
}

fn main() {
    let a = Article { title: String::from("Rust is great"), author: String::from("Alice") };
    println!("{}", a.summarise());

    let v1 = Vec2 { x: 1.0, y: 2.0 };
    let v2 = Vec2 { x: 3.0, y: 4.0 };
    println!("{:?} + {:?} = {:?}", v1, v2, v1 + v2);
    println!("{:?} + (10, 0) = {:?}", v1, v1 + (10.0, 0.0));

    let w = Wrapper(vec!["hello".to_string(), "world".to_string()]);
    println!("{w}");
}
"#,
                ),
                (
                    "macros",
                    r#"#![allow(dead_code)]
// Ch 19 – Macros
// macro_rules! defines declarative macros via pattern matching on token trees.
// They expand at compile time — zero runtime cost, can be variadic.

// Recreate vec![] to show how it works
macro_rules! my_vec {
    ()                              => { Vec::new() };
    ( $( $x:expr ),+ $(,)? )        => {{ let mut v = Vec::new(); $( v.push($x); )+ v }};
}

// Implement a trait for multiple types in one shot
macro_rules! impl_display {
    ( $( $t:ty ),+ ) => {
        $(
            impl std::fmt::Display for $t {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{:?}", self)
                }
            }
        )+
    };
}

#[derive(Debug)] struct Metres(f64);
#[derive(Debug)] struct Kilograms(f64);
impl_display!(Metres, Kilograms);

// A simple assertion macro with a descriptive message
macro_rules! check {
    ($cond:expr) => {
        if $cond { println!("  ✓ {}", stringify!($cond)); }
        else     { println!("  ✗ {} (FAILED)", stringify!($cond)); }
    };
}

fn main() {
    let v1: Vec<i32> = my_vec![];
    let v2 = my_vec![1, 2, 3];
    let v3 = my_vec!["a", "b", "c",]; // trailing comma OK
    println!("v1={v1:?}");
    println!("v2={v2:?}");
    println!("v3={v3:?}");

    println!("{}", Metres(5.2));
    println!("{}", Kilograms(70.0));

    check!(2 + 2 == 4);
    check!(v2.len() == 3);
    check!(v1.is_empty());
    check!(2 + 2 == 5); // intentional failure to show the output
}
"#,
                ),
            ],
        ),
        // ── Chapter 20: Final Project – Web Server ─────────────────────────────
        (
            "ch20_web_server",
            "",
            vec![
                (
                    "threadpool",
                    r#"#![allow(dead_code)]
// Ch 20 – Thread Pool (the heart of the final project)
// The web server in Ch 20 is built around a ThreadPool that dispatches
// incoming HTTP connections to a fixed pool of worker threads.
// Here we implement and exercise the ThreadPool without the HTTP layer.
// See cli_guide.rs to build and run the full web server.

use std::sync::{Arc, Mutex, mpsc};
use std::thread;

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender:  Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    pub fn new(size: usize) -> Self {
        assert!(size > 0, "pool size must be > 0");
        let (tx, rx) = mpsc::channel::<Job>();
        let rx = Arc::new(Mutex::new(rx));
        let workers = (0..size).map(|id| Worker::new(id, Arc::clone(&rx))).collect();
        Self { workers, sender: Some(tx) }
    }

    pub fn execute<F: FnOnce() + Send + 'static>(&self, f: F) {
        self.sender.as_ref().unwrap().send(Box::new(f)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take()); // closing the channel signals workers to exit
        for w in &mut self.workers {
            if let Some(t) = w.thread.take() { t.join().unwrap(); }
        }
        println!("All workers shut down cleanly.");
    }
}

struct Worker { id: usize, thread: Option<thread::JoinHandle<()>> }

impl Worker {
    fn new(id: usize, rx: Arc<Mutex<mpsc::Receiver<Job>>>) -> Self {
        let thread = thread::spawn(move || loop {
            match rx.lock().unwrap().recv() {
                Ok(job)  => { println!("  worker {id}: running job"); job(); }
                Err(_)   => { println!("  worker {id}: shutting down"); break; }
            }
        });
        Self { id, thread: Some(thread) }
    }
}

fn main() {
    println!("Creating pool of 4 workers...");
    let pool = ThreadPool::new(4);

    for i in 0..8 {
        pool.execute(move || {
            println!("  job {i} finished");
        });
    }

    thread::sleep(std::time::Duration::from_millis(100));
    println!("Dropping pool — graceful shutdown:");
    // pool is dropped here → Drop impl runs → all workers exit cleanly
}
"#,
                ),
                (
                    "cli_guide",
                    r#"// Ch 20 – Multithreaded Web Server: CLI Guide
// ══════════════════════════════════════════════════════════════════
//  Chapter 20 builds a real HTTP server that listens on a TCP port.
//  A TcpListener that actually accepts connections would block this
//  playground forever — run it in a terminal instead.
//
//  ── BUILD THE WEB SERVER ───────────────────────────────────────
//
//  $ cargo new hello_server
//  $ cd hello_server
//  # Copy the final src/main.rs and src/lib.rs from the book:
//  # https://doc.rust-lang.org/book/ch20-02-multithreaded.html
//
//  Create two response files:
//  $ echo "<h1>Hello!</h1>"         > hello.html
//  $ echo "<h1>404 Not Found</h1>" > 404.html
//
//  $ cargo run
//  # Server listens on 127.0.0.1:7878
//
//  In another terminal (or your browser):
//  $ curl http://127.0.0.1:7878/
//  $ curl http://127.0.0.1:7878/sleep   # slow endpoint
//
//  Stop the server with Ctrl-C.
//
//  ── WHAT THE CHAPTER COVERS ────────────────────────────────────
//  • TcpListener and TcpStream
//  • Parsing a raw HTTP request
//  • Writing an HTTP response
//  • Building a ThreadPool (see threadpool.rs in this project!)
//  • Graceful shutdown via Drop
//
//  ── READ MORE ──────────────────────────────────────────────────
//  https://doc.rust-lang.org/book/ch20-00-final-project-a-web-server.html
// ══════════════════════════════════════════════════════════════════

fn main() {
    println!("Ch 20 – Multithreaded Web Server");
    println!();
    println!("The thread pool implementation is in threadpool.rs — run that with ⌘R.");
    println!("For the full HTTP server, jump to your terminal:");
    println!();
    println!("  cargo new hello_server && cd hello_server");
    println!("  # follow https://doc.rust-lang.org/book/ch20-01-single-threaded.html");
    println!("  cargo run");
    println!("  # then visit http://127.0.0.1:7878 in your browser");
}
"#,
                ),
            ],
        ),
    ]; // end of chapters vec

    for (chapter, extra_deps, playgrounds) in &chapters {
        let project_path = pdir.join(chapter);
        if project_path.exists() {
            continue;
        } // idempotent — skip if already there

        let bin = project_path.join("src").join("bin");
        std::fs::create_dir_all(&bin).map_err(|e| format!("seed {chapter}: {e}"))?;

        let cargo = format!(
            "[package]\nname = \"{chapter}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n# Add dependencies here — every playground can use them.\n[dependencies]\n{extra_deps}",
        );
        std::fs::write(project_path.join("Cargo.toml"), &cargo)
            .map_err(|e| format!("seed {chapter} Cargo.toml: {e}"))?;
        let content_dir = project_path.join("content");
        std::fs::create_dir_all(&content_dir)
            .map_err(|e| format!("seed {chapter} content/: {e}"))?;
        std::fs::write(content_dir.join("attribution.md"), ATTRIBUTION_MD)
            .map_err(|e| format!("seed {chapter} attribution.md: {e}"))?;

        for (pg, code) in playgrounds {
            std::fs::write(bin.join(format!("{pg}.rs")), code)
                .map_err(|e| format!("seed {chapter}/{pg}: {e}"))?;
        }
        created.push(chapter.to_string());
    }

    Ok(created)
}
