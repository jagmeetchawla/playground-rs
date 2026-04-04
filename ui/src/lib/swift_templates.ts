import type { Template } from './templates'

export const SWIFT_TEMPLATES: Template[] = [
  {
    id: 'swift_blank',
    name: 'Blank',
    description: 'Empty Swift playground',
    lang: 'swift' as any,
    code: `import Foundation

`,
  },
  {
    id: 'swift_hello',
    name: 'Hello World',
    description: 'Print to stdout',
    lang: 'swift' as any,
    code: `import Foundation

print("Hello, world!")
`,
  },
  {
    id: 'swift_structs',
    name: 'Structs & Methods',
    description: 'Value types with methods',
    lang: 'swift' as any,
    code: `import Foundation

struct Point {
    var x: Double
    var y: Double

    func distance(to other: Point) -> Double {
        let dx = x - other.x
        let dy = y - other.y
        return (dx * dx + dy * dy).squareRoot()
    }
}

let a = Point(x: 1.0, y: 2.0)
let b = Point(x: 4.0, y: 6.0)
print("Distance: \\(a.distance(to: b))")
`,
  },
  {
    id: 'swift_enums',
    name: 'Enums & Pattern Matching',
    description: 'Enums with associated values',
    lang: 'swift' as any,
    code: `import Foundation

enum Shape {
    case circle(radius: Double)
    case rectangle(width: Double, height: Double)
    case triangle(base: Double, height: Double)

    var area: Double {
        switch self {
        case .circle(let r):
            return Double.pi * r * r
        case .rectangle(let w, let h):
            return w * h
        case .triangle(let b, let h):
            return 0.5 * b * h
        }
    }

    var description: String {
        switch self {
        case .circle(let r): return "Circle(r=\\(r))"
        case .rectangle(let w, let h): return "Rect(\\(w)x\\(h))"
        case .triangle(let b, let h): return "Tri(b=\\(b),h=\\(h))"
        }
    }
}

let shapes: [Shape] = [
    .circle(radius: 5),
    .rectangle(width: 4, height: 6),
    .triangle(base: 3, height: 8),
]

for shape in shapes {
    print("\\(shape.description) -> area = \\(String(format: "%.2f", shape.area))")
}
`,
  },
  {
    id: 'swift_optionals',
    name: 'Optionals & Guard',
    description: 'Safe unwrapping patterns',
    lang: 'swift' as any,
    code: `import Foundation

func greet(_ name: String?) -> String {
    guard let name = name, !name.isEmpty else {
        return "Hello, stranger!"
    }
    return "Hello, \\(name)!"
}

print(greet("Alice"))
print(greet(nil))
print(greet(""))

// Optional chaining
let numbers: [Int]? = [1, 2, 3, 4, 5]
if let count = numbers?.count {
    print("Got \\(count) numbers")
}

// Nil coalescing
let username: String? = nil
let display = username ?? "anonymous"
print("User: \\(display)")
`,
  },
  {
    id: 'swift_collections',
    name: 'Collections',
    description: 'Arrays, dictionaries, and sets',
    lang: 'swift' as any,
    code: `import Foundation

// Arrays
var fruits = ["apple", "banana", "cherry"]
fruits.append("date")
print("Fruits: \\(fruits)")

// Map & filter
let lengths = fruits.map { $0.count }
let long = fruits.filter { $0.count > 5 }
print("Lengths: \\(lengths)")
print("Long names: \\(long)")

// Dictionary
var scores: [String: Int] = ["Alice": 95, "Bob": 87, "Charlie": 92]
scores["Diana"] = 98

for (name, score) in scores.sorted(by: { $0.value > $1.value }) {
    print("  \\(name): \\(score)")
}

// Set
let primes: Set = [2, 3, 5, 7, 11, 13]
let evens: Set = [2, 4, 6, 8, 10, 12]
print("Prime & even: \\(primes.intersection(evens))")
`,
  },
]
