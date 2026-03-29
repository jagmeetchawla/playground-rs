fn main() {
    println!("Running closures playground");

    let add = |x| x + 1;
    println!("{}", add(5)); // 6

    let sum = |x, y| x + y;
    println!("{}", sum(5, 6));

    let sum_of_three = |x: i32, y: i32, z: i32| x + y + z;
    println!("{}", sum_of_three(1, 2, 3));

    let v = vec![1, 2, 3];
    let result: Vec<_> = v
        .iter()
        .map(|x| x * 2)
        .collect();
    println!("{:?}", result); // [2, 4, 6]

    let x = 10;
    let f = |y| x + y; // borrows x
    println!("{}", f(5));

    let mut count = 0;
    let mut inc = || {
        count += 1;
    };
    inc();
    inc();
    println!("{}", count); // 2

    let s = String::from("hello");
    let f = move || {
        println!("{}", s);
    };
    f();
}