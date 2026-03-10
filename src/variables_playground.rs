pub fn run() {
    // --- variables playground ---

    let x = 5;
    let mut y = 10;
    y += x;

    println!("x = {x}");
    println!("y (mutable) = {y}");

    // shadowing
    let x = x * 2;
    println!("x (shadowed) = {x}");

    // types
    let pi: f64 = std::f64::consts::PI;
    let greeting: &str = "hello";
    println!("{greeting}, pi is approximately {pi}");
}
