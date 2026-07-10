export type Template = {
  id: string
  name: string
  description: string
  code: string
  lang?: string
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
      { name: 'serde_json', version: '1' },
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
  // ── Modern Rust showcase (v0.4+) ────────────────────────────────────────
  // These templates highlight features that stabilised in relatively recent
  // Rust versions. They compile on Rust 1.85+ (edition 2024, our MIN_RUST),
  // and are especially expressive on the newer channels the picker lets
  // users install.
  {
    id: 'let_chains',
    name: 'Let-chains (Rust 1.88+)',
    description: 'if let with && for readable pattern matching',
    code: `//! Let-chains stabilised in Rust 1.88 (2025). They let you combine
//! \`if let\` pattern binding with boolean conditions via \`&&\` — turning
//! rightward-drifting nested \`if let Some(x) = foo { if x > 0 {\` chains
//! into a single flat expression.

fn parse_port(s: &str) -> Option<u16> {
    s.parse().ok()
}

fn main() {
    let inputs = ["8080", "http", "22", "0", "65535", "not-a-port"];

    for s in inputs {
        // Old style would nest \`if let Some(p) = parse_port(s)\` and then
        // \`if p > 0 && p < 1024\`. With let-chains it flattens beautifully:
        if let Some(port) = parse_port(s) && port > 0 && port < 1024 {
            println!("{s:>12} → privileged port {port}");
        } else if let Some(port) = parse_port(s) && port >= 1024 {
            println!("{s:>12} → user port {port}");
        } else {
            println!("{s:>12} → not a usable port");
        }
    }

    // Works in \`while let\` too — pull items only while they satisfy a check.
    let mut queue = vec![Some(1), Some(2), Some(3), None, Some(4)];
    while let Some(head) = queue.pop() && let Some(n) = head {
        println!("popped {n}");
    }
    println!("stopped at: {:?}", queue);
}
`,
  },
  {
    id: 'let_else',
    name: 'Let-else patterns',
    description: 'Early-return refactoring with let ... else { ... }',
    code: `//! \`let ... else\` (stabilised in Rust 1.65) is a cleaner alternative to
//! matching-then-returning. It binds a pattern in the current scope, and
//! the \`else\` branch must diverge (\`return\`, \`continue\`, \`break\`,
//! \`panic!\`, or \`std::process::exit\`) — the compiler enforces this.

/// Parse "key=value" or return None. Traditional \`match\` version below
/// for comparison — the let-else version scales far better as pipelines grow.
fn parse_pair_match(s: &str) -> Option<(String, String)> {
    match s.split_once('=') {
        Some((k, v)) => match k.trim() {
            "" => None,
            k => Some((k.to_string(), v.trim().to_string())),
        },
        None => None,
    }
}

fn parse_pair_let_else(s: &str) -> Option<(String, String)> {
    let Some((k, v)) = s.split_once('=') else { return None; };
    let k = k.trim();
    if k.is_empty() { return None; }
    Some((k.to_string(), v.trim().to_string()))
}

fn main() {
    let inputs = ["  name = alice  ", "empty=", "no-equals", "=lonely"];

    for s in inputs {
        // Both versions produce the same result — let-else is just flatter.
        let a = parse_pair_match(s);
        let b = parse_pair_let_else(s);
        assert_eq!(a, b);
        println!("{s:>22} → {b:?}");
    }
}
`,
  },
  {
    id: 'modern_std',
    name: 'Modern std tour',
    description: 'Recent std combinators: is_some_and, inspect, array windows',
    code: `//! A short tour of small std additions that landed in Rust 1.70–1.80.
//! Each of these replaces a common awkward pattern with something clearer.
//!
//! - Option::is_some_and / Result::is_ok_and  (1.70)
//! - Option::inspect / Result::inspect         (1.76)
//! - <[T]>::windows and <[T]>::chunks_exact    (evergreen but underused)

fn main() {
    let scores = vec![82, 91, 88, 75, 94, 68];

    // ── is_some_and: replace \`.map().unwrap_or(false)\` for boolean checks ──
    let best = scores.iter().max();
    if best.is_some_and(|&s| s >= 90) {
        println!("Great result! Top score: {}", best.unwrap());
    }

    // ── inspect: peek at a value in a pipeline without breaking it ─────
    // Old style needed a let-binding to log a value mid-pipeline. inspect()
    // lets you slot it in without disturbing the chain.
    let total: i32 = scores
        .iter()
        .inspect(|s| println!("  visiting {s}"))
        .filter(|&&s| s >= 70)
        .sum();
    println!("Total of passing scores: {total}");

    // ── windows: slide a fixed-size look at consecutive elements ────────
    // Great for pairwise diffs, moving averages, or detecting patterns.
    let series = [3, 7, 4, 9, 2, 6, 8];
    print!("Differences: ");
    for w in series.windows(2) {
        print!("{} ", w[1] - w[0]);
    }
    println!();

    // ── chunks_exact: iterate in fixed-size groups, ignoring a partial tail ─
    let bytes = [0xDE_u8, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE];
    print!("Hex pairs: ");
    for pair in bytes.chunks_exact(2) {
        print!("{:02X}{:02X} ", pair[0], pair[1]);
    }
    println!();
}
`,
  },
]

// ── Native C templates ────────────────────────────────────────────────────────

export const C_TEMPLATES: Template[] = [
  {
    id: 'c_blank',
    name: 'Blank',
    description: 'Empty C playground',
    lang: 'c',
    code: `#include <stdio.h>

int main() {

    return 0;
}
`,
  },
  {
    id: 'c_hello',
    name: 'Hello World',
    description: 'Variables, printf formatting',
    lang: 'c',
    code: `#include <stdio.h>

int main() {
    const char *name = "C";
    int year = 1972;

    printf("Hello, %s!\\n", name);
    printf("%s was created in %d.\\n", name, year);
    printf("Result: %d\\n", 2 + 2);

    // Formatted output
    double pi = 3.14159;
    printf("Pi is approximately %.2f\\n", pi);
    printf("%10s | %10s\\n", "Item", "Price");
    printf("%10s | %10.2f\\n", "Coffee", 4.5);
    printf("%10s | %10.2f\\n", "Muffin", 3.25);

    return 0;
}
`,
  },
  {
    id: 'c_input',
    name: 'CLI Input',
    description: 'Read from stdin, parse input',
    lang: 'c',
    code: `#include <stdio.h>
#include <stdlib.h>

int main() {
    char name[64];
    int age;

    printf("What is your name? ");
    fflush(stdout);
    if (fgets(name, sizeof(name), stdin)) {
        // Strip trailing newline
        for (int i = 0; name[i]; i++) {
            if (name[i] == '\\n') { name[i] = '\\0'; break; }
        }
    }

    printf("How old are you? ");
    fflush(stdout);
    if (scanf("%d", &age) == 1) {
        printf("Hello %s, you will be %d next year!\\n", name, age + 1);
    } else {
        printf("That's not a valid number, %s!\\n", name);
    }

    return 0;
}
`,
  },
  {
    id: 'c_structs',
    name: 'Structs',
    description: 'Structs, typedef, function pointers',
    lang: 'c',
    code: `#include <stdio.h>
#include <math.h>

typedef struct {
    double x;
    double y;
} Point;

double distance(Point a, Point b) {
    double dx = a.x - b.x;
    double dy = a.y - b.y;
    return sqrt(dx * dx + dy * dy);
}

typedef struct {
    const char *name;
    double dimension1;
    double dimension2;
    double (*area)(double, double);
} Shape;

double circle_area(double r, double _unused) {
    (void)_unused;
    return 3.14159265 * r * r;
}

double rect_area(double w, double h) {
    return w * h;
}

double tri_area(double b, double h) {
    return 0.5 * b * h;
}

int main() {
    Point a = {0.0, 0.0};
    Point b = {3.0, 4.0};
    printf("Distance from (%.1f,%.1f) to (%.1f,%.1f): %.2f\\n",
           a.x, a.y, b.x, b.y, distance(a, b));

    Shape shapes[] = {
        {"Circle(5)",      5.0, 0.0, circle_area},
        {"Rect(4x6)",      4.0, 6.0, rect_area},
        {"Triangle(3x8)",  3.0, 8.0, tri_area},
    };
    int n = sizeof(shapes) / sizeof(shapes[0]);

    for (int i = 0; i < n; i++) {
        printf("%s -> area = %.2f\\n", shapes[i].name,
               shapes[i].area(shapes[i].dimension1, shapes[i].dimension2));
    }

    return 0;
}
`,
  },
  {
    id: 'c_arrays',
    name: 'Arrays & Strings',
    description: 'Arrays, string functions, sorting',
    lang: 'c',
    code: `#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int compare_ints(const void *a, const void *b) {
    return (*(int *)a - *(int *)b);
}

int main() {
    // Array basics
    int numbers[] = {42, 17, 93, 5, 68, 31, 84, 12};
    int n = sizeof(numbers) / sizeof(numbers[0]);

    printf("Original: ");
    for (int i = 0; i < n; i++) printf("%d ", numbers[i]);
    printf("\\n");

    qsort(numbers, n, sizeof(int), compare_ints);

    printf("Sorted:   ");
    for (int i = 0; i < n; i++) printf("%d ", numbers[i]);
    printf("\\n");

    // Sum and average
    int sum = 0;
    for (int i = 0; i < n; i++) sum += numbers[i];
    printf("Sum: %d, Average: %.1f\\n\\n", sum, (double)sum / n);

    // String operations
    char greeting[64] = "Hello";
    strcat(greeting, ", World!");
    printf("Greeting: %s (length: %zu)\\n", greeting, strlen(greeting));

    char copy[64];
    strcpy(copy, greeting);
    printf("Copy: %s\\n", copy);
    printf("Compare: %d (0 = equal)\\n", strcmp(greeting, copy));

    return 0;
}
`,
  },
  {
    id: 'c_pointers',
    name: 'Pointers & Memory',
    description: 'Pointers, malloc, linked list',
    lang: 'c',
    code: `#include <stdio.h>
#include <stdlib.h>

// A simple singly-linked list
typedef struct Node {
    int value;
    struct Node *next;
} Node;

Node *node_new(int value) {
    Node *n = malloc(sizeof(Node));
    n->value = value;
    n->next = NULL;
    return n;
}

void list_push(Node **head, int value) {
    Node *n = node_new(value);
    n->next = *head;
    *head = n;
}

void list_print(Node *head) {
    for (Node *cur = head; cur; cur = cur->next) {
        printf("%d", cur->value);
        if (cur->next) printf(" -> ");
    }
    printf("\\n");
}

void list_free(Node *head) {
    while (head) {
        Node *next = head->next;
        free(head);
        head = next;
    }
}

int main() {
    // Pointer basics
    int x = 42;
    int *p = &x;
    printf("x = %d, *p = %d, address = %p\\n", x, *p, (void *)p);

    *p = 100;
    printf("After *p = 100: x = %d\\n\\n", x);

    // Dynamic array
    int n = 5;
    int *arr = malloc(n * sizeof(int));
    for (int i = 0; i < n; i++) arr[i] = (i + 1) * 10;

    printf("Dynamic array: ");
    for (int i = 0; i < n; i++) printf("%d ", arr[i]);
    printf("\\n\\n");
    free(arr);

    // Linked list
    Node *list = NULL;
    for (int i = 1; i <= 5; i++) list_push(&list, i * 10);

    printf("Linked list: ");
    list_print(list);
    list_free(list);

    return 0;
}
`,
  },
  {
    id: 'c_file_io',
    name: 'File I/O',
    description: 'Read and write files using content folder',
    lang: 'c',
    code: `#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main() {
    // Get the content folder path (set by the playground runner)
    const char *content_dir = getenv("PLAYGROUND_CONTENT");
    if (!content_dir) content_dir = ".";

    char path[256];
    snprintf(path, sizeof(path), "%s/scores.csv", content_dir);

    // Write a file
    FILE *f = fopen(path, "w");
    if (!f) { perror("fopen write"); return 1; }
    fprintf(f, "Name,Score\\n");
    fprintf(f, "Alice,95\\n");
    fprintf(f, "Bob,87\\n");
    fprintf(f, "Charlie,92\\n");
    fclose(f);
    printf("Wrote %s\\n\\n", path);

    // Read it back and process
    f = fopen(path, "r");
    if (!f) { perror("fopen read"); return 1; }

    char line[128];
    int total = 0, count = 0;

    fgets(line, sizeof(line), f); // skip header
    printf("Parsed results:\\n");
    while (fgets(line, sizeof(line), f)) {
        char name[64];
        int score;
        if (sscanf(line, "%[^,],%d", name, &score) == 2) {
            printf("  %s: %d\\n", name, score);
            total += score;
            count++;
        }
    }
    fclose(f);

    if (count > 0) {
        printf("\\nAverage: %.1f\\n", (double)total / count);
    }

    return 0;
}
`,
  },
  {
    id: 'c_sqlite',
    name: 'SQLite',
    description: 'In-memory database with sqlite3',
    lang: 'c',
    code: `#include <stdio.h>
#include <sqlite3.h>

static int print_row(void *unused, int ncols, char **values, char **names) {
    (void)unused;
    for (int i = 0; i < ncols; i++) {
        printf("  %s = %s", names[i], values[i] ? values[i] : "NULL");
        if (i < ncols - 1) printf(",");
    }
    printf("\\n");
    return 0;
}

int main() {
    sqlite3 *db;
    char *err = NULL;

    if (sqlite3_open(":memory:", &db) != SQLITE_OK) {
        fprintf(stderr, "Cannot open database: %s\\n", sqlite3_errmsg(db));
        return 1;
    }

    // Create table and insert data
    const char *sql =
        "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, score INTEGER);"
        "INSERT INTO users (name, score) VALUES ('Alice', 95);"
        "INSERT INTO users (name, score) VALUES ('Bob', 87);"
        "INSERT INTO users (name, score) VALUES ('Charlie', 92);"
        "INSERT INTO users (name, score) VALUES ('Diana', 78);";

    if (sqlite3_exec(db, sql, NULL, NULL, &err) != SQLITE_OK) {
        fprintf(stderr, "SQL error: %s\\n", err);
        sqlite3_free(err);
        sqlite3_close(db);
        return 1;
    }

    printf("All users:\\n");
    sqlite3_exec(db, "SELECT * FROM users ORDER BY score DESC;", print_row, NULL, NULL);

    printf("\\nAverage score:\\n");
    sqlite3_exec(db, "SELECT AVG(score) as avg_score FROM users;", print_row, NULL, NULL);

    sqlite3_close(db);
    return 0;
}
`,
  },
]

// ── Native C++ templates ──────────────────────────────────────────────────────

export const CPP_TEMPLATES: Template[] = [
  {
    id: 'cpp_blank',
    name: 'Blank',
    description: 'Empty C++ playground',
    lang: 'cpp',
    code: `#include <iostream>

int main() {

    return 0;
}
`,
  },
  {
    id: 'cpp_hello',
    name: 'Hello World',
    description: 'Variables, streams, formatting',
    lang: 'cpp',
    code: `#include <iostream>
#include <iomanip>
#include <string>

int main() {
    std::string name = "C++";
    int year = 1985;

    std::cout << "Hello, " << name << "!" << std::endl;
    std::cout << name << " was created in " << year << "." << std::endl;
    std::cout << "Result: " << 2 + 2 << std::endl;

    // Formatted output
    double pi = 3.14159;
    std::cout << "Pi is approximately " << std::fixed << std::setprecision(2) << pi << std::endl;
    std::cout << std::setw(10) << "Item" << " | " << std::setw(10) << "Price" << std::endl;
    std::cout << std::setw(10) << "Coffee" << " | " << std::setw(10) << 4.5 << std::endl;
    std::cout << std::setw(10) << "Muffin" << " | " << std::setw(10) << 3.25 << std::endl;

    return 0;
}
`,
  },
  {
    id: 'cpp_input',
    name: 'CLI Input',
    description: 'Read from stdin, parse input',
    lang: 'cpp',
    code: `#include <iostream>
#include <string>

int main() {
    std::string name;
    std::cout << "What is your name? " << std::flush;
    std::getline(std::cin, name);

    std::cout << "How old are you? " << std::flush;
    int age;
    if (std::cin >> age) {
        std::cout << "Hello " << name << ", you will be " << age + 1 << " next year!" << std::endl;
    } else {
        std::cout << "That's not a valid number, " << name << "!" << std::endl;
    }

    return 0;
}
`,
  },
  {
    id: 'cpp_classes',
    name: 'Classes',
    description: 'Classes, inheritance, polymorphism',
    lang: 'cpp',
    code: `#include <iostream>
#include <string>
#include <vector>
#include <memory>
#include <cmath>

struct Point {
    double x, y;

    double distance_to(const Point &other) const {
        double dx = x - other.x;
        double dy = y - other.y;
        return std::sqrt(dx * dx + dy * dy);
    }
};

class Shape {
public:
    virtual ~Shape() = default;
    virtual std::string name() const = 0;
    virtual double area() const = 0;
};

class Circle : public Shape {
    double radius;
public:
    Circle(double r) : radius(r) {}
    std::string name() const override { return "Circle(" + std::to_string(radius) + ")"; }
    double area() const override { return M_PI * radius * radius; }
};

class Rectangle : public Shape {
    double w, h;
public:
    Rectangle(double w, double h) : w(w), h(h) {}
    std::string name() const override { return "Rect(" + std::to_string(w) + "x" + std::to_string(h) + ")"; }
    double area() const override { return w * h; }
};

int main() {
    Point a{0.0, 0.0}, b{3.0, 4.0};
    std::cout << "Distance: " << a.distance_to(b) << std::endl;

    std::vector<std::unique_ptr<Shape>> shapes;
    shapes.push_back(std::make_unique<Circle>(5.0));
    shapes.push_back(std::make_unique<Rectangle>(4.0, 6.0));

    for (const auto &s : shapes) {
        std::cout << s->name() << " -> area = " << s->area() << std::endl;
    }

    return 0;
}
`,
  },
  {
    id: 'cpp_containers',
    name: 'Containers',
    description: 'vector, map, algorithms, ranges',
    lang: 'cpp',
    code: `#include <iostream>
#include <vector>
#include <map>
#include <algorithm>
#include <numeric>
#include <string>
#include <sstream>

int main() {
    // Vector + algorithms
    std::vector<int> numbers = {1, 2, 3, 4, 5, 6, 7, 8, 9, 10};

    std::vector<int> even_squares;
    for (int n : numbers) {
        if (n % 2 == 0) even_squares.push_back(n * n);
    }

    std::cout << "Even squares: ";
    for (int n : even_squares) std::cout << n << " ";
    std::cout << std::endl;

    int sum = std::accumulate(numbers.begin(), numbers.end(), 0);
    std::cout << "Sum: " << sum << std::endl;

    // Map — word frequency counter
    std::string text = "the cat sat on the mat the cat";
    std::map<std::string, int> freq;
    std::istringstream iss(text);
    std::string word;
    while (iss >> word) freq[word]++;

    // Sort by frequency
    std::vector<std::pair<std::string, int>> sorted(freq.begin(), freq.end());
    std::sort(sorted.begin(), sorted.end(),
              [](const auto &a, const auto &b) { return b.second < a.second; });

    std::cout << "\\nWord frequencies:" << std::endl;
    for (const auto &[w, c] : sorted) {
        std::cout << "  " << w << " : " << c << std::endl;
    }

    // Partition: even/odd
    std::vector<int> evens, odds;
    for (int n : numbers) {
        (n % 2 == 0 ? evens : odds).push_back(n);
    }
    std::cout << "\\nEvens: ";
    for (int n : evens) std::cout << n << " ";
    std::cout << "\\nOdds:  ";
    for (int n : odds) std::cout << n << " ";
    std::cout << std::endl;

    return 0;
}
`,
  },
  {
    id: 'cpp_smart_ptrs',
    name: 'Smart Pointers',
    description: 'unique_ptr, shared_ptr, RAII',
    lang: 'cpp',
    code: `#include <iostream>
#include <memory>
#include <string>
#include <vector>

class Resource {
    std::string name;
public:
    Resource(const std::string &n) : name(n) {
        std::cout << "  [+] " << name << " acquired" << std::endl;
    }
    ~Resource() {
        std::cout << "  [-] " << name << " released" << std::endl;
    }
    void use() const { std::cout << "  Using " << name << std::endl; }
};

int main() {
    // unique_ptr — exclusive ownership
    std::cout << "=== unique_ptr ===" << std::endl;
    {
        auto r = std::make_unique<Resource>("FileHandle");
        r->use();
        // auto copy = r;  // ERROR: can't copy unique_ptr
        auto moved = std::move(r);
        moved->use();
    } // automatically released here
    std::cout << std::endl;

    // shared_ptr — reference counted
    std::cout << "=== shared_ptr ===" << std::endl;
    {
        auto shared = std::make_shared<Resource>("Connection");
        std::cout << "  ref count: " << shared.use_count() << std::endl;
        {
            auto copy = shared;
            std::cout << "  ref count: " << shared.use_count() << std::endl;
            copy->use();
        }
        std::cout << "  ref count: " << shared.use_count() << std::endl;
    }
    std::cout << std::endl;

    // Vector of unique_ptrs
    std::cout << "=== vector<unique_ptr> ===" << std::endl;
    {
        std::vector<std::unique_ptr<Resource>> resources;
        resources.push_back(std::make_unique<Resource>("GPU"));
        resources.push_back(std::make_unique<Resource>("Audio"));
        for (const auto &r : resources) r->use();
    }

    return 0;
}
`,
  },
  {
    id: 'cpp_file_io',
    name: 'File I/O',
    description: 'Read and write files using content folder',
    lang: 'cpp',
    code: `#include <iostream>
#include <fstream>
#include <sstream>
#include <string>
#include <cstdlib>

int main() {
    // Get the content folder path (set by the playground runner)
    const char *env = std::getenv("PLAYGROUND_CONTENT");
    std::string content_dir = env ? env : ".";
    std::string path = content_dir + "/scores.csv";

    // Write a file
    {
        std::ofstream out(path);
        out << "Name,Score\\n";
        out << "Alice,95\\n";
        out << "Bob,87\\n";
        out << "Charlie,92\\n";
    }
    std::cout << "Wrote " << path << "\\n\\n";

    // Read it back and process
    std::ifstream in(path);
    std::string line;
    int total = 0, count = 0;

    std::getline(in, line); // skip header
    std::cout << "Parsed results:\\n";
    while (std::getline(in, line)) {
        auto comma = line.find(',');
        if (comma != std::string::npos) {
            std::string name = line.substr(0, comma);
            int score = std::stoi(line.substr(comma + 1));
            std::cout << "  " << name << ": " << score << "\\n";
            total += score;
            count++;
        }
    }

    if (count > 0) {
        std::cout << "\\nAverage: " << (double)total / count << "\\n";
    }

    return 0;
}
`,
  },
  {
    id: 'cpp_sqlite',
    name: 'SQLite',
    description: 'In-memory database with sqlite3',
    lang: 'cpp',
    code: `#include <iostream>
#include <sqlite3.h>

static int print_row(void *, int ncols, char **values, char **names) {
    for (int i = 0; i < ncols; i++) {
        std::cout << "  " << names[i] << " = " << (values[i] ? values[i] : "NULL");
        if (i < ncols - 1) std::cout << ",";
    }
    std::cout << std::endl;
    return 0;
}

int main() {
    sqlite3 *db;
    char *err = nullptr;

    if (sqlite3_open(":memory:", &db) != SQLITE_OK) {
        std::cerr << "Cannot open database: " << sqlite3_errmsg(db) << std::endl;
        return 1;
    }

    const char *sql =
        "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, score INTEGER);"
        "INSERT INTO users (name, score) VALUES ('Alice', 95);"
        "INSERT INTO users (name, score) VALUES ('Bob', 87);"
        "INSERT INTO users (name, score) VALUES ('Charlie', 92);"
        "INSERT INTO users (name, score) VALUES ('Diana', 78);";

    if (sqlite3_exec(db, sql, nullptr, nullptr, &err) != SQLITE_OK) {
        std::cerr << "SQL error: " << err << std::endl;
        sqlite3_free(err);
        sqlite3_close(db);
        return 1;
    }

    std::cout << "All users:" << std::endl;
    sqlite3_exec(db, "SELECT * FROM users ORDER BY score DESC;", print_row, nullptr, nullptr);

    std::cout << "\\nAverage score:" << std::endl;
    sqlite3_exec(db, "SELECT AVG(score) as avg_score FROM users;", print_row, nullptr, nullptr);

    sqlite3_close(db);
    return 0;
}
`,
  },
]
