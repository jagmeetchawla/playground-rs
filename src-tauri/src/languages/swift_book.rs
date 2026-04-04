use std::path::Path;

/// Seed Swift Book chapter projects into the given projects directory.
/// Returns the list of newly created project names.
pub fn seed_book(pdir: &Path) -> Result<Vec<String>, String> {
    let mut created: Vec<String> = Vec::new();

    const ATTRIBUTION_MD: &str = "\
# Source: The Swift Programming Language

These playground examples are based on the curriculum of:

**\"The Swift Programming Language\"**
by Apple Inc.

| | |
|---|---|
| Book (free online) | https://docs.swift.org/swift-book/ |
| Source repository  | https://github.com/swiftlang/swift-book |
| License            | Apache-2.0 with Runtime Library Exception |
| Copyright          | Apple Inc. and the Swift project authors |

## Notes

The playground code is original educational Swift written to illustrate the
same concepts covered in each chapter. It is not verbatim source from the
book. Rustic Playground is not affiliated with or endorsed by Apple Inc.
or the Swift project.
";

    // (project_name, [(playground_name, code)])
    #[allow(clippy::type_complexity)]
    let chapters: Vec<(&str, Vec<(&str, &str)>)> = vec![
        // ── Chapter 1: The Basics ───────────────────────────────────────────
        (
            "swift_ch01_basics",
            vec![
                (
                    "hello.swift",
                    r#"// Ch 1 – Hello, World!
// Every Swift program can start executing at the top level.
// print() writes to stdout with a newline.

import Foundation

let language = "Swift"
let year = 2014
print("Hello, world!")
print("\(language) was released in \(year).")

// String interpolation with expressions
print("2 + 3 = \(2 + 3)")

// Multi-line strings
let haiku = """
    Code compiles at last,
    Runtime errors now appear —
    Such is the coder.
    """
print(haiku)
"#,
                ),
                (
                    "variables.swift",
                    r#"// Ch 1 – Variables, Constants, and Type Annotations
// let = constant (immutable), var = variable (mutable).
// Swift is type-safe with powerful type inference.

import Foundation

// Constants — prefer let whenever possible
let pi = 3.14159
let greeting: String = "Hello"
// pi = 3.0  // ERROR: cannot assign to 'let' constant

// Variables
var score = 0
score += 10
print("Score: \(score)")

// Type annotations
var temperature: Double = 98.6
var name: String = "Swift"
var isActive: Bool = true

// Multiple declarations
var x = 0, y = 0, z = 0

// Type conversion (no implicit conversion in Swift)
let intVal = 42
let doubleVal = 3.14
let sum = Double(intVal) + doubleVal
print("Sum: \(sum)")

// Tuples — group multiple values
let http404 = (404, "Not Found")
print("Status: \(http404.0) — \(http404.1)")

let (code, message) = http404
print("Code: \(code), Message: \(message)")

// Named tuple elements
let person = (name: "Alice", age: 30)
print("\(person.name) is \(person.age)")
"#,
                ),
                (
                    "optionals.swift",
                    r#"// Ch 1 – Optionals
// Optionals handle the absence of a value. A core Swift safety feature.

import Foundation

// An optional might contain a value or nil
var greeting: String? = "Hello"
var empty: String? = nil

// Optional binding with if-let
if let g = greeting {
    print("Greeting: \(g)")
} else {
    print("No greeting")
}

// Guard let — early exit pattern
func greet(_ name: String?) {
    guard let name = name else {
        print("No name provided")
        return
    }
    print("Hello, \(name)!")
}
greet("Alice")
greet(nil)

// Nil-coalescing operator (??)
let nickname: String? = nil
let display = nickname ?? "Anonymous"
print("User: \(display)")

// Optional chaining
let names: [String]? = ["Alice", "Bob", "Charlie"]
let count = names?.count ?? 0
print("Count: \(count)")

// Forced unwrapping (use sparingly — crashes on nil!)
let definiteValue: String? = "I exist"
print(definiteValue!)

// Implicitly unwrapped optionals (for values set once after init)
let assumedValue: String! = "Set during setup"
let implicitValue: String = assumedValue
print(implicitValue)
"#,
                ),
            ],
        ),
        // ── Chapter 2: Control Flow ─────────────────────────────────────────
        (
            "swift_ch02_control_flow",
            vec![
                (
                    "conditionals.swift",
                    r#"// Ch 2 – Conditionals (if, switch)
// Swift's switch is powerful — no implicit fallthrough, pattern matching built in.

import Foundation

// Basic if-else
let temp = 72
if temp > 80 {
    print("It's hot")
} else if temp > 60 {
    print("It's pleasant")
} else {
    print("It's cool")
}

// Switch — must be exhaustive
let char: Character = "a"
switch char {
case "a", "e", "i", "o", "u":
    print("\(char) is a vowel")
case "b"..."z":
    print("\(char) is a consonant")
default:
    print("Not a lowercase letter")
}

// Switch with value binding
let point = (2, 0)
switch point {
case (let x, 0):
    print("On x-axis at \(x)")
case (0, let y):
    print("On y-axis at \(y)")
case let (x, y) where x == y:
    print("On diagonal at \(x)")
case let (x, y):
    print("At (\(x), \(y))")
}

// Switch with compound cases
let number = 42
switch number {
case 0:
    print("Zero")
case 1...9:
    print("Single digit")
case 10...99:
    print("Double digit")
default:
    print("Large number")
}
"#,
                ),
                (
                    "loops.swift",
                    r#"// Ch 2 – Loops (for-in, while, repeat-while)
// Swift uses for-in instead of C-style for loops.

import Foundation

// For-in with range
print("Counting:")
for i in 1...5 {
    print("  \(i)")
}

// Half-open range (excludes upper bound)
let names = ["Alice", "Bob", "Charlie"]
for i in 0..<names.count {
    print("  \(i): \(names[i])")
}

// For-in with array directly
for name in names {
    print("  Hello, \(name)!")
}

// Stride — like a C for loop with step
print("Even numbers:")
for n in stride(from: 0, to: 10, by: 2) {
    print("  \(n)")
}

// While loop
var countdown = 5
while countdown > 0 {
    print("  \(countdown)...")
    countdown -= 1
}
print("  Liftoff!")

// Repeat-while (like do-while in C)
var roll: Int
repeat {
    roll = Int.random(in: 1...6)
    print("  Rolled \(roll)")
} while roll != 6
print("  Got a six!")

// Labelled loops with break
outerLoop: for i in 1...3 {
    for j in 1...3 {
        if i == 2 && j == 2 {
            print("Breaking at (\(i), \(j))")
            break outerLoop
        }
        print("  (\(i), \(j))")
    }
}
"#,
                ),
            ],
        ),
        // ── Chapter 3: Functions ────────────────────────────────────────────
        (
            "swift_ch03_functions",
            vec![
                (
                    "functions.swift",
                    r#"// Ch 3 – Functions
// Swift functions have expressive argument labels and support default values.

import Foundation

// Basic function
func greet(person: String) -> String {
    return "Hello, \(person)!"
}
print(greet(person: "World"))

// Multiple return values with tuples
func minMax(array: [Int]) -> (min: Int, max: Int)? {
    guard !array.isEmpty else { return nil }
    var lo = array[0], hi = array[0]
    for val in array {
        if val < lo { lo = val }
        if val > hi { hi = val }
    }
    return (lo, hi)
}
if let result = minMax(array: [3, 1, 4, 1, 5, 9]) {
    print("Min: \(result.min), Max: \(result.max)")
}

// Argument labels vs parameter names
func move(from start: String, to end: String) {
    print("Moving from \(start) to \(end)")
}
move(from: "London", to: "Paris")

// Omitting argument labels with _
func multiply(_ a: Int, _ b: Int) -> Int { a * b }
print("3 x 4 = \(multiply(3, 4))")

// Default parameter values
func power(_ base: Int, exponent: Int = 2) -> Int {
    var result = 1
    for _ in 0..<exponent { result *= base }
    return result
}
print("5^2 = \(power(5))")
print("2^10 = \(power(2, exponent: 10))")

// Variadic parameters
func average(_ numbers: Double...) -> Double {
    let total = numbers.reduce(0, +)
    return total / Double(numbers.count)
}
print("Average: \(average(1, 2, 3, 4, 5))")
"#,
                ),
                (
                    "closures.swift",
                    r#"// Ch 3 – Closures
// Closures are self-contained blocks of functionality.
// They capture values from the surrounding context.

import Foundation

// Closure expression syntax
let numbers = [5, 2, 8, 1, 9, 3]
let sorted = numbers.sorted(by: { (a: Int, b: Int) -> Bool in
    return a < b
})
print("Sorted: \(sorted)")

// Shorthand: inferred types, implicit return, shorthand names
let sortedShort = numbers.sorted { $0 < $1 }
print("Short:  \(sortedShort)")

// Trailing closure syntax
let doubled = numbers.map { $0 * 2 }
print("Doubled: \(doubled)")

// Chaining higher-order functions
let result = numbers
    .filter { $0 > 3 }
    .map { $0 * $0 }
    .sorted()
print("Filtered, squared, sorted: \(result)")

// Capturing values
func makeCounter() -> () -> Int {
    var count = 0
    return {
        count += 1
        return count
    }
}
let counter = makeCounter()
print("Count: \(counter()), \(counter()), \(counter())")

// Closure as a function parameter
func apply(_ value: Int, transform: (Int) -> Int) -> Int {
    return transform(value)
}
let tripled = apply(7) { $0 * 3 }
print("Tripled: \(tripled)")
"#,
                ),
            ],
        ),
        // ── Chapter 4: Collections ──────────────────────────────────────────
        (
            "swift_ch04_collections",
            vec![
                (
                    "arrays.swift",
                    r#"// Ch 4 – Arrays and Sets
// Swift collections are type-safe and offer rich APIs.

import Foundation

// Arrays — ordered collections of same-type values
var fruits = ["Apple", "Banana", "Cherry"]
print("Fruits: \(fruits)")
print("Count: \(fruits.count)")

// Modifying arrays
fruits.append("Date")
fruits.insert("Avocado", at: 1)
fruits.remove(at: 2)  // removes "Banana"
print("Modified: \(fruits)")

// Array operations
let numbers = [3, 1, 4, 1, 5, 9, 2, 6]
print("Sum: \(numbers.reduce(0, +))")
print("Max: \(numbers.max()!)")
print("Sorted: \(numbers.sorted())")
print("Reversed: \(Array(numbers.reversed()))")

// Iterating with index
for (i, fruit) in fruits.enumerated() {
    print("  \(i): \(fruit)")
}

// Array slicing
let slice = numbers[2...5]
print("Slice [2...5]: \(Array(slice))")

// Sets — unordered collections of unique values
var colors: Set<String> = ["Red", "Green", "Blue"]
colors.insert("Yellow")
colors.insert("Red")  // duplicate — ignored
print("Colors: \(colors.sorted())")

// Set operations
let primary: Set = ["Red", "Green", "Blue"]
let warm: Set = ["Red", "Orange", "Yellow"]
print("Intersection: \(primary.intersection(warm))")
print("Union: \(primary.union(warm).sorted())")
print("Difference: \(primary.subtracting(warm))")
"#,
                ),
                (
                    "dictionaries.swift",
                    r#"// Ch 4 – Dictionaries
// Key-value pairs with fast lookup. Keys must be Hashable.

import Foundation

// Creating dictionaries
var capitals: [String: String] = [
    "France": "Paris",
    "Japan": "Tokyo",
    "Brazil": "Brasilia",
]

// Accessing values (returns Optional)
if let capital = capitals["France"] {
    print("France's capital: \(capital)")
}

// Adding and updating
capitals["Germany"] = "Berlin"
capitals["Brazil"] = "Brasilia"

// Removing
capitals["Japan"] = nil
print("Capitals: \(capitals)")

// Iterating
print("All capitals:")
for (country, city) in capitals.sorted(by: { $0.key < $1.key }) {
    print("  \(country): \(city)")
}

// Dictionary with default values
let text = "hello world hello swift hello"
var wordCount: [String: Int] = [:]
for word in text.split(separator: " ") {
    let w = String(word)
    wordCount[w, default: 0] += 1
}
print("Word counts:")
for (word, count) in wordCount.sorted(by: { $0.value > $1.value }) {
    print("  \(word): \(count)")
}

// Transforming dictionaries
let scores = ["Alice": 85, "Bob": 92, "Charlie": 78]
let grades = scores.mapValues { score -> String in
    switch score {
    case 90...100: return "A"
    case 80..<90: return "B"
    case 70..<80: return "C"
    default: return "F"
    }
}
print("Grades: \(grades)")
"#,
                ),
            ],
        ),
        // ── Chapter 5: Structs and Classes ──────────────────────────────────
        (
            "swift_ch05_structs_classes",
            vec![
                (
                    "structs.swift",
                    r#"// Ch 5 – Structs (Value Types)
// Structs are copied when assigned or passed. Prefer structs over classes.

import Foundation

// Basic struct
struct Point {
    var x: Double
    var y: Double

    // Computed property
    var magnitude: Double {
        return (x * x + y * y).squareRoot()
    }

    // Mutating method (required for value types)
    mutating func translate(dx: Double, dy: Double) {
        x += dx
        y += dy
    }

    func distance(to other: Point) -> Double {
        let dx = x - other.x
        let dy = y - other.y
        return (dx * dx + dy * dy).squareRoot()
    }
}

var p1 = Point(x: 3, y: 4)  // memberwise initializer
print("Point: (\(p1.x), \(p1.y)), magnitude: \(p1.magnitude)")

p1.translate(dx: 1, dy: -1)
print("Translated: (\(p1.x), \(p1.y))")

let p2 = Point(x: 0, y: 0)
print("Distance to origin: \(p1.distance(to: p2))")

// Value semantics — copy on assignment
var p3 = p1
p3.x = 100
print("p1.x = \(p1.x), p3.x = \(p3.x)")  // independent copies

// Struct with custom init
struct Temperature {
    var celsius: Double

    var fahrenheit: Double {
        celsius * 9 / 5 + 32
    }

    init(celsius: Double) { self.celsius = celsius }
    init(fahrenheit: Double) { self.celsius = (fahrenheit - 32) * 5 / 9 }
}

let boiling = Temperature(celsius: 100)
print("Boiling: \(boiling.celsius)C = \(boiling.fahrenheit)F")

let body = Temperature(fahrenheit: 98.6)
print("Body: \(body.fahrenheit)F = \(String(format: "%.1f", body.celsius))C")
"#,
                ),
                (
                    "classes.swift",
                    r#"// Ch 5 – Classes (Reference Types)
// Classes support inheritance and are reference types (shared, not copied).

import Foundation

// Base class
class Animal {
    let name: String
    var sound: String

    init(name: String, sound: String) {
        self.name = name
        self.sound = sound
    }

    func speak() -> String {
        return "\(name) says \(sound)!"
    }
}

// Subclass
class Dog: Animal {
    var tricks: [String] = []

    init(name: String) {
        super.init(name: name, sound: "Woof")
    }

    func learn(_ trick: String) {
        tricks.append(trick)
    }

    override func speak() -> String {
        return "\(name) barks: \(sound)!"
    }
}

let dog = Dog(name: "Rex")
dog.learn("sit")
dog.learn("shake")
print(dog.speak())
print("Tricks: \(dog.tricks)")

// Reference semantics — shared instance
let sameDog = dog
sameDog.sound = "WOOF"
print("Original: \(dog.sound)")  // also "WOOF" — same reference

// Identity check
print("Same object? \(dog === sameDog)")  // true

// Struct vs Class comparison
print("\n--- Value vs Reference ---")
struct PointS { var x: Int }
class PointC { var x: Int; init(x: Int) { self.x = x } }

var sv = PointS(x: 1)
var sv2 = sv
sv2.x = 99
print("Struct: sv.x=\(sv.x), sv2.x=\(sv2.x)")    // 1, 99

let cv = PointC(x: 1)
let cv2 = cv
cv2.x = 99
print("Class:  cv.x=\(cv.x), cv2.x=\(cv2.x)")     // 99, 99
"#,
                ),
            ],
        ),
        // ── Chapter 6: Enumerations ─────────────────────────────────────────
        (
            "swift_ch06_enums",
            vec![(
                "enums.swift",
                r#"// Ch 6 – Enumerations
// Swift enums are first-class types with associated values and methods.

import Foundation

// Basic enum
enum Direction {
    case north, south, east, west
}
var heading = Direction.north
heading = .east  // type inferred after first assignment
print("Heading: \(heading)")

// Enum with raw values
enum Planet: Int {
    case mercury = 1, venus, earth, mars, jupiter, saturn, uranus, neptune
}
let earth = Planet.earth
print("Earth is planet #\(earth.rawValue)")

if let planet = Planet(rawValue: 4) {
    print("Planet 4 is \(planet)")
}

// Enum with String raw values
enum Suit: String, CaseIterable {
    case hearts = "♥️"
    case diamonds = "♦️"
    case clubs = "♣️"
    case spades = "♠️"
}
for suit in Suit.allCases {
    print("  \(suit): \(suit.rawValue)")
}

// Enum with associated values — the powerful part
enum Barcode {
    case upc(Int, Int, Int, Int)
    case qr(String)
}

let product1 = Barcode.upc(8, 85909, 51226, 3)
let product2 = Barcode.qr("ABCDEF")

func describe(_ code: Barcode) {
    switch code {
    case .upc(let ns, let mfr, let prod, let check):
        print("UPC: \(ns)-\(mfr)-\(prod)-\(check)")
    case .qr(let data):
        print("QR: \(data)")
    }
}
describe(product1)
describe(product2)

// Enum with methods
enum Coin: Double {
    case penny = 0.01
    case nickel = 0.05
    case dime = 0.10
    case quarter = 0.25

    func dollarString() -> String {
        return String(format: "$%.2f", self.rawValue)
    }
}

let coins: [Coin] = [.quarter, .dime, .nickel, .penny, .quarter]
let total = coins.reduce(0.0) { $0 + $1.rawValue }
print("Coins: \(coins.map { $0.dollarString() })")
print("Total: \(String(format: "$%.2f", total))")
"#,
            )],
        ),
        // ── Chapter 7: Protocols ────────────────────────────────────────────
        (
            "swift_ch07_protocols",
            vec![(
                "protocols.swift",
                r#"// Ch 7 – Protocols and Extensions
// Protocols define a blueprint of methods and properties.
// Extensions add functionality to existing types.

import Foundation

// Define a protocol
protocol Describable {
    var description: String { get }
    func summary() -> String
}

// Conform to the protocol
struct City: Describable {
    let name: String
    let population: Int

    var description: String {
        "\(name) (pop. \(population))"
    }

    func summary() -> String {
        "City of \(name) with \(population) residents"
    }
}

let paris = City(name: "Paris", population: 2_161_000)
print(paris.description)
print(paris.summary())

// Protocol with default implementation via extension
protocol Greetable {
    var name: String { get }
}

extension Greetable {
    func greet() -> String {
        "Hello, I'm \(name)!"
    }
}

struct Person: Greetable {
    let name: String
}

let alice = Person(name: "Alice")
print(alice.greet())

// Extensions on existing types
extension Int {
    var isEven: Bool { self % 2 == 0 }
    var squared: Int { self * self }

    func times(_ action: () -> Void) {
        for _ in 0..<self { action() }
    }
}

print("4 is even? \(4.isEven)")
print("5 squared: \(5.squared)")
3.times { print("  Hip hip hooray!") }

// Protocol-oriented polymorphism
protocol Shape {
    func area() -> Double
}

struct Circle: Shape {
    let radius: Double
    func area() -> Double { .pi * radius * radius }
}

struct Rectangle: Shape {
    let width: Double, height: Double
    func area() -> Double { width * height }
}

let shapes: [Shape] = [Circle(radius: 5), Rectangle(width: 4, height: 6)]
for shape in shapes {
    print("Area: \(String(format: "%.2f", shape.area()))")
}
print("Total: \(String(format: "%.2f", shapes.reduce(0) { $0 + $1.area() }))")
"#,
            )],
        ),
        // ── Chapter 8: Error Handling ───────────────────────────────────────
        (
            "swift_ch08_error_handling",
            vec![(
                "errors.swift",
                r#"// Ch 8 – Error Handling
// Swift uses do-try-catch for expressive error handling.

import Foundation

// Define error types with enums
enum ATMError: Error {
    case insufficientFunds(balance: Double, requested: Double)
    case invalidAmount
    case cardBlocked
}

// A function that can throw
struct BankAccount {
    var balance: Double

    mutating func withdraw(_ amount: Double) throws -> Double {
        guard amount > 0 else {
            throw ATMError.invalidAmount
        }
        guard amount <= balance else {
            throw ATMError.insufficientFunds(balance: balance, requested: amount)
        }
        balance -= amount
        return amount
    }
}

// Using do-try-catch
var account = BankAccount(balance: 100.0)

do {
    let amount = try account.withdraw(30)
    print("Withdrew $\(amount). Balance: $\(account.balance)")
} catch ATMError.insufficientFunds(let balance, let requested) {
    print("Not enough funds: have $\(balance), need $\(requested)")
} catch ATMError.invalidAmount {
    print("Invalid amount")
} catch {
    print("Unexpected error: \(error)")
}

// try? — converts to optional (nil on error)
let result = try? account.withdraw(200)
print("try? result: \(result as Any)")  // nil

// try! — force (crashes on error, use only when guaranteed)
let safe = try! account.withdraw(10)
print("Force withdrew: $\(safe)")

// Defer — cleanup that always runs
func processFile() {
    print("Opening file...")
    defer { print("Closing file (defer)") }
    defer { print("Flushing buffer (defer)") }
    print("Processing file...")
    // defers run in reverse order when scope exits
}
processFile()
"#,
            )],
        ),
    ]; // end of chapters vec

    for (chapter, playgrounds) in &chapters {
        let project_path = pdir.join(chapter);
        if project_path.exists() {
            continue;
        } // idempotent — skip if already there

        std::fs::create_dir_all(&project_path).map_err(|e| format!("seed {chapter}: {e}"))?;

        // Write swift manifest with source tag
        let mut manifest = crate::rustic_manifest::new_swift_manifest();
        manifest.project.source = "swift_book".to_string();
        manifest.project.readonly = true;
        crate::rustic_manifest::write_manifest(&project_path, &manifest)?;

        // Content directory + attribution
        let content_dir = project_path.join("content");
        std::fs::create_dir_all(&content_dir)
            .map_err(|e| format!("seed {chapter} content/: {e}"))?;
        std::fs::write(content_dir.join("attribution.md"), ATTRIBUTION_MD)
            .map_err(|e| format!("seed {chapter} attribution.md: {e}"))?;

        for (pg, code) in playgrounds {
            std::fs::write(project_path.join(pg), code)
                .map_err(|e| format!("seed {chapter}/{pg}: {e}"))?;
        }
        created.push(chapter.to_string());
    }

    Ok(created)
}
