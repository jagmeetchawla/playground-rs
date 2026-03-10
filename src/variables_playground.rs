pub fn run() {
    // --- variables playground ---

    let x = 5;
    let mut y = 10;
    y += x;

    println!("x = {x}");
    println!("y = {y}");

    // shadowing
    let x = x * 2;
    println!("x (shadowed) = {x}");

    // types
    let pi: f64 = 3.14159;
    let greeting: &str = "hello";
    println!("{greeting}, pi is approximately {pi}");
}
