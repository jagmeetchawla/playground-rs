pub fn run() {
    println!("Chapter 4 Playground");

    let mut s = String::from("Hello");
    s.push_str(", world!");
    println!("{}", s);

    let x = 5;
    let y = x;
    println!("x = {}, y = {}", x, y);

    let s1 = String::from("hello");
    let s2 = s1.clone();
    let s3 = s1;

    //println!("s1 = {}, s2 = {}", s1, s2);
    println!("s1 = {}, s2 = {}", s2, s3);

    take_ownership(s2);
    //println!("s2 = {}", s2);

    let s4 = give_ownership();
    println!("s4 = {}", s4);

    let s5 = take_and_give_back(s4);
    println!("s5 = {}", s5);

    let (s6, length) = calculate_length(String::from("hello"));
    println!("s6 = {}, length = {}", s6, length);
    println!("length = {}", calculate_length2(&s6));

    let s7 = String::from("hello");
    let length = calculate_length2(&s7);
    println!("length = {}", length);

    let mut s8 = String::from("hello");
    change_string(&mut s8);
    println!("s8 = {}", s8);

    let r1 = & s8;
    let r2 = & s8;
    println!("r1 = {}, r2 = {}", r1, r2);

    let r3 = &mut s8;
    //let r4 = &mut s8;
    //let r5 = & s8;
    println!("r3 = {}", r3);
    //println!("r1 = {}, r2 = {}", r1, r2);

    let mut s9 = String::from("hello");
    {
        let r6 = &mut s9;
        println!("r6 = {}", r6);
    }

    let r7 = &mut s9;
    println!("r7 = {}", r7);
}

fn take_ownership(some_string: String) {
    println!("{}", some_string);
}

fn give_ownership() -> String {
    String::from("ownership")
}

fn take_and_give_back(a_string: String) -> String {
    println!("Taking and giving back: {}", a_string);
    a_string
}

fn calculate_length(s: String) -> (String, usize) {
    let length = s.len();
    (s, length)
}

fn calculate_length2(s: &str) -> usize {
    s.len()
}

fn change_string(some_string: &mut String) {
    some_string.push_str(", world");
}