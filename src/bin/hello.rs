fn main() {
    println!("Hello from the playground!");

    let nums: Vec<i32> = (1..=5).collect();
    let sum: i32 = nums.iter().sum();
    println!("Sum of 1..=5 is {}", sum);
}
