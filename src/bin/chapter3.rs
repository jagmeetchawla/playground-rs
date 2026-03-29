fn main() {
    // --- Rust Book Chapter 3 playground ---

    // variable shadowing
    let spaces = "   ";
    let spaces = spaces.len();
    println!("spaces = {}", spaces);

    // blocks as expressions
    let x = { 5 + 10 };
    println!("x = {}", x);

    let y = {
        let x = 3;
        x + 1
    };
    println!("y = {}", y);

    // if expressions
    let condition = true;
    let x = if condition { 5 } else { 6 };
    println!("x = {}", x);

    // loop as expression
    let mut counter = 0;
    let result = loop {
        counter += 1;
        if counter == 10 { break counter * 2; }
    };
    println!("result = {}", result);

    // for loop
    for item in 0..10 {
        println!("item = {}", item);
    }

    // for loop reversed
    for item in (0..10).rev() {
        println!("item = {}", item);
    }

    // for loop on an array
    let a = [10, 20, 30, 40, 50];
    for item in a.iter() {
        println!("item = {}", item);
    }

    // for loop on vector
    let mut v = vec![10, 20, 30, 40, 50];
    v.push(60);
    for item in v.iter() {
        println!("item = {}", item);
    }

    // while loop
    let mut counter = 3;
    while counter != 0 {
        println!("counter = {}", counter);
        counter -= 1;
    }

    // while loop on an array
    let a = [10, 20, 30, 40, 50];
    let mut index = 0;
    while index < a.len() {
        println!("a[{}] = {}", index, a[index]);
        index += 1;
    }
}