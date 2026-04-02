export type Template = {
  id: string
  name: string
  description: string
  code: string
  deps?: { name: string; version: string }[]
}

export const TEMPLATES: Template[] = [
  {
    id: 'blank',
    name: 'Blank',
    description: 'Empty playground',
    code: `fn main() {

}
`,
  },
  {
    id: 'hello',
    name: 'Hello World',
    description: 'Variables, formatting, printing',
    code: `fn main() {
    let name = "Rust";
    let year = 2024;

    println!("Hello, {name}!");
    println!("{name} has been growing since {year}.");
    println!("Result: {}", 2 + 2);

    // Formatted output
    let pi = 3.14159;
    println!("Pi is approximately {pi:.2}");
    println!("{:>10} | {:>10}", "Item", "Price");
    println!("{:>10} | {:>10.2}", "Coffee", 4.5);
    println!("{:>10} | {:>10.2}", "Muffin", 3.25);
}
`,
  },
  {
    id: 'cli_input',
    name: 'CLI Input',
    description: 'Read from stdin, parse input',
    code: `use std::io::{self, Write};

fn main() {
    // Prompt the user
    print!("What is your name? ");
    io::stdout().flush().unwrap();

    let mut name = String::new();
    io::stdin().read_line(&mut name).unwrap();
    let name = name.trim();

    print!("How old are you? ");
    io::stdout().flush().unwrap();

    let mut age_str = String::new();
    io::stdin().read_line(&mut age_str).unwrap();

    match age_str.trim().parse::<u32>() {
        Ok(age) => println!("Hello {name}, you will be {} next year!", age + 1),
        Err(_) => println!("That's not a valid number, {name}!"),
    }
}
`,
  },
  {
    id: 'structs_enums',
    name: 'Structs & Enums',
    description: 'Types, impl blocks, pattern matching',
    code: `#[derive(Debug)]
struct Point {
    x: f64,
    y: f64,
}

impl Point {
    fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    fn distance_to(&self, other: &Point) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

#[derive(Debug)]
enum Shape {
    Circle(f64),
    Rectangle(f64, f64),
    Triangle(f64, f64),
}

impl Shape {
    fn area(&self) -> f64 {
        match self {
            Shape::Circle(r) => std::f64::consts::PI * r * r,
            Shape::Rectangle(w, h) => w * h,
            Shape::Triangle(b, h) => 0.5 * b * h,
        }
    }
}

fn main() {
    let a = Point::new(0.0, 0.0);
    let b = Point::new(3.0, 4.0);
    println!("Distance from {a:?} to {b:?}: {:.2}", a.distance_to(&b));

    let shapes = vec![
        Shape::Circle(5.0),
        Shape::Rectangle(4.0, 6.0),
        Shape::Triangle(3.0, 8.0),
    ];

    for shape in &shapes {
        println!("{:?} -> area = {:.2}", shape, shape.area());
    }
}
`,
  },
  {
    id: 'error_handling',
    name: 'Error Handling',
    description: 'Result, ? operator, custom errors',
    code: `use std::fmt;
use std::num::ParseIntError;

// Custom error type
#[derive(Debug)]
enum AppError {
    ParseError(ParseIntError),
    ValidationError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::ParseError(e) => write!(f, "Parse error: {e}"),
            AppError::ValidationError(msg) => write!(f, "Validation error: {msg}"),
        }
    }
}

impl From<ParseIntError> for AppError {
    fn from(e: ParseIntError) -> Self {
        AppError::ParseError(e)
    }
}

fn parse_age(input: &str) -> Result<u32, AppError> {
    let age: u32 = input.trim().parse()?; // ? converts ParseIntError via From
    if age > 150 {
        return Err(AppError::ValidationError("Age seems unrealistic".into()));
    }
    Ok(age)
}

fn main() {
    let inputs = vec!["25", "abc", "200", "42"];

    for input in inputs {
        match parse_age(input) {
            Ok(age) => println!("'{input}' -> Valid age: {age}"),
            Err(e) => println!("'{input}' -> Error: {e}"),
        }
    }
}
`,
  },
  {
    id: 'collections',
    name: 'Collections',
    description: 'Vec, HashMap, iterators, combinators',
    code: `use std::collections::HashMap;

fn main() {
    // Vec + iterator combinators
    let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    let even_squares: Vec<i32> = numbers
        .iter()
        .filter(|n| *n % 2 == 0)
        .map(|n| n * n)
        .collect();
    println!("Even squares: {even_squares:?}");

    let sum: i32 = numbers.iter().sum();
    println!("Sum: {sum}");

    // HashMap — word frequency counter
    let text = "the cat sat on the mat the cat";
    let mut freq: HashMap<&str, usize> = HashMap::new();

    for word in text.split_whitespace() {
        *freq.entry(word).or_insert(0) += 1;
    }

    // Sort by frequency (descending)
    let mut sorted: Vec<_> = freq.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));

    println!("\\nWord frequencies:");
    for (word, count) in &sorted {
        println!("  {word:>6} : {count}");
    }

    // Chaining: group numbers by even/odd
    let (evens, odds): (Vec<&i32>, Vec<&i32>) = numbers.iter().partition(|n| *n % 2 == 0);
    println!("\\nEvens: {evens:?}");
    println!("Odds:  {odds:?}");
}
`,
  },
  {
    id: 'file_io',
    name: 'File I/O',
    description: 'Read and write files using content folder',
    code: `use std::{env, fs};

fn main() {
    // Get the content folder path (set by the playground runner)
    let content_dir = env::var("PLAYGROUND_CONTENT").unwrap_or_else(|_| ".".into());

    // Write a file
    let data = "Name,Score\\nAlice,95\\nBob,87\\nCharlie,92\\n";
    let path = format!("{content_dir}/scores.csv");
    fs::write(&path, data).expect("Failed to write file");
    println!("Wrote scores.csv");

    // Read it back and process
    let contents = fs::read_to_string(&path).expect("Failed to read file");
    println!("\\nRaw contents:\\n{contents}");

    // Parse CSV manually
    println!("Parsed results:");
    let mut total = 0;
    let mut count = 0;
    for line in contents.lines().skip(1) {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() == 2 {
            let name = parts[0];
            let score: i32 = parts[1].parse().unwrap_or(0);
            println!("  {name}: {score}");
            total += score;
            count += 1;
        }
    }
    if count > 0 {
        println!("\\nAverage: {:.1}", total as f64 / count as f64);
    }
}
`,
  },
  {
    id: 'concurrency',
    name: 'Concurrency',
    description: 'Threads, channels, Arc/Mutex',
    code: `use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

fn main() {
    // Channel example: producer/consumer
    let (tx, rx) = mpsc::channel();

    let producer = thread::spawn(move || {
        for i in 1..=5 {
            tx.send(format!("Message {i}")).unwrap();
            thread::sleep(Duration::from_millis(100));
        }
    });

    let consumer = thread::spawn(move || {
        for msg in rx {
            println!("Received: {msg}");
        }
    });

    producer.join().unwrap();
    consumer.join().unwrap();

    // Shared state with Arc<Mutex<T>>
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    for id in 0..4 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            for _ in 0..10 {
                let mut num = counter.lock().unwrap();
                *num += 1;
            }
            println!("Thread {id} done");
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().unwrap();
    }
    println!("\\nFinal count: {}", *counter.lock().unwrap());
}
`,
  },
  {
    id: 'async_tokio',
    name: 'Async',
    description: 'Tokio runtime, async/await, spawn',
    deps: [{ name: 'tokio', version: '{ version = "1", features = ["full"] }' }],
    code: `use std::time::Duration;

async fn fetch_data(id: u32) -> String {
    // Simulate async work
    tokio::time::sleep(Duration::from_millis(100 * id as u64)).await;
    format!("Data from task {id}")
}

async fn run_tasks() {
    // Spawn concurrent tasks
    let mut handles = vec![];
    for i in 1..=5 {
        handles.push(tokio::spawn(async move {
            let result = fetch_data(i).await;
            println!("{result}");
            result
        }));
    }

    // Await all results
    let mut results = vec![];
    for handle in handles {
        results.push(handle.await.unwrap());
    }
    println!("\\nAll {len} tasks completed!", len = results.len());
}

#[tokio::main]
async fn main() {
    println!("Starting async tasks...\\n");
    run_tasks().await;
}
`,
  },
  {
    id: 'web_request',
    name: 'Web Request',
    description: 'HTTP client with reqwest',
    deps: [
      { name: 'reqwest', version: '{ version = "0.12", features = ["blocking"] }' },
    ],
    code: `fn main() {
    println!("Fetching data from httpbin.org...\\n");

    // Simple GET request (blocking)
    match reqwest::blocking::get("https://httpbin.org/json") {
        Ok(response) => {
            println!("Status: {}", response.status());

            // Print headers
            println!("\\nHeaders:");
            for (key, value) in response.headers() {
                if key.as_str().starts_with("content") {
                    println!("  {key}: {}", value.to_str().unwrap_or("?"));
                }
            }

            // Print body
            match response.text() {
                Ok(body) => {
                    println!("\\nBody (first 300 chars):");
                    println!("{}", &body[..body.len().min(300)]);
                }
                Err(e) => println!("Failed to read body: {e}"),
            }
        }
        Err(e) => println!("Request failed: {e}"),
    }
}
`,
  },
  {
    id: 'serde_json',
    name: 'Serde JSON',
    description: 'Serialize and deserialize structs',
    deps: [
      { name: 'serde', version: '{ version = "1", features = ["derive"] }' },
      { name: 'serde_json', version: '"1"' },
    ],
    code: `use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct User {
    name: String,
    age: u32,
    email: String,
    #[serde(default)]
    active: bool,
}

fn main() {
    // Serialize struct to JSON
    let user = User {
        name: "Alice".into(),
        age: 30,
        email: "alice@example.com".into(),
        active: true,
    };

    let json = serde_json::to_string_pretty(&user).unwrap();
    println!("Serialized:\\n{json}\\n");

    // Deserialize JSON to struct
    let json_str = r#"{
        "name": "Bob",
        "age": 25,
        "email": "bob@example.com"
    }"#;

    let bob: User = serde_json::from_str(json_str).unwrap();
    println!("Deserialized: {bob:#?}");
    println!("Active (default): {}", bob.active);

    // Work with dynamic JSON
    let data: serde_json::Value = serde_json::json!({
        "users": [
            { "name": "Charlie", "score": 95 },
            { "name": "Diana", "score": 88 },
        ]
    });

    println!("\\nDynamic JSON:");
    if let Some(users) = data["users"].as_array() {
        for u in users {
            println!("  {} scored {}", u["name"], u["score"]);
        }
    }
}
`,
  },
]
