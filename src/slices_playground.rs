pub fn run() {
    let s = "hello world";

    let slice1 = &s[0..5];
    println!("{}", slice1);
    let slice2 = &s[6..11];
    println!("{}", slice2);

    let hello = first_word(s);
    println!("{}", hello);

    let world = second_word(s);
    println!("{}", world);

    let a = [1, 2, 3, 4, 5];
    let slice = &a[1..3];
    println!("{:?}", slice);
}

fn first_word(s: &str) -> &str {
    let bytes = s.as_bytes();
    let first_byte_index = bytes.iter().position(|&x| x == b' ').unwrap();
    &s[0..first_byte_index]
}

fn second_word(s: &str) -> &str {
    let bytes = s.as_bytes();
    let first_byte_index = bytes.iter().position(|&x| x == b' ').unwrap();
    &s[first_byte_index + 1..]
}