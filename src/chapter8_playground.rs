use unicode_segmentation::UnicodeSegmentation;
use std::collections::HashMap;
pub fn run() {

    let mut v:Vec<i32> = Vec::new();
    v.push(5);
    v.push(6);
    v.push(7);
    v.push(8);
    v.push(9);
    println!("{:?}", v);
    v.pop();
    println!("{:?}", v);
    v.remove(0);
    println!("{:?}", v);
    v.swap(0, 1);
    println!("{:?}", v);
    v.swap_remove(1);
    println!("{:?}", v);

    let mut v2 = vec![1, 2, 3, 4, 5];
    let mut v3 = v2.split_off(2);
    println!("{:?}", v2);
    println!("{:?}", v3);

    v2.push(10);
    println!("{:?}", v2);
    v2.append(&mut v3);
    println!("{:?}", v2);
    println!("{:?}", v3);
    let v4 = vec![8,1];
    v2.extend(v4);
    println!("{:?}", v2);

    v2.drain(1..3);
    println!("{:?}", v2);

    v2.retain(|&x| x % 2 == 0);
    println!("{:?}", v2);
    v2.reverse();
    println!("{:?}", v2);
    v2.sort();
    println!("{:?}", v2);

    let v5 = vec![1,2,3,4,5,6,7,8,9];
    println!("{:?}", v5.iter().count());
    for (i, v) in v5.iter().enumerate() {
        println!("Index: {}, Value: {}", i, v);
    }
    v5.iter().for_each(|i| println!("{}", i));
    let v6 = v5.iter().map(|i| i * 2).collect::<Vec<i32>>();
    println!("{:?}", v6);
    let v6:i32 /* Type */ = v5.iter().sum();
    let total: i32 = v5.iter()
        .map(|x| x)
        .sum();
    let result = v5.iter()
        .map(|x| x * 1)
        .reduce(|a, b| a + b);
    let result2 = v5.iter().fold(0, |acc, x| acc + x);
    println!("{:?}", v6);
    println!("{:?}", total);
    println!("{:?}", result);
    println!("{:?}", result2);

    let arr = [10, 20, 30, 40];
    let v_arr = arr.to_vec();
    println!("{:?}", v_arr);

    let v_arr2 = (1..100).collect::<Vec<i32>>();
    println!("{:?}", v_arr2);

    let mut v = vec![1,2,3,4,5];
    let e1 = &v[2];
    // v.push(6); won't compile
    println!("{:?}", e1);
    let e2 = v.get(2);
    if let Some(e) = e2 {
        println!("{:?}", e);
    }
    let e3 = v.get(10);
    if let None = e3 {
        println!("None");
    }
    v.push(6);
    println!("{:?}", v);

    for i in &mut v {
        *i += 50;
    }
    println!("{:?}", v);

    let a = v[3];
    let b = &v[3];
    // drop(v);
    println!("{}", a);
    println!("{}", b);

    let s1 = String::from("hello,");
    let s2 = String::from("world!");
    let s3 = s1 + &s2;
    println!("{}", s3);
    //println!("{:?}", s1); //s1 moved out to s3
    let mut s4 = s3.clone();
    s4.push_str("!");
    println!("{}", s4);

    let s5 = format!("{}.  {}\n{}\n{}.  {}", 1, s3, "------", 2, s2);
    println!("{}", s5);

    let s = "hello";
    println!("{}", s.len());

    let s = "नमस्ते";
    println!("{}", s.len());

    let s = "é";
    println!("{}", s.len());

    let s = "🚀";
    println!("{}", s.len());

    let s = "नमस्ते";

    for c in s.chars() {
        println!("{}", c);
    }

    println!("bytes: {}", s.len());
    println!("chars: {}", s.chars().count());

    let graphemes: Vec<&str> = s.graphemes(true).collect();

    println!("graphemes: {}", graphemes.len());

    for g in graphemes {
        println!("{}", g);
    }

    let mut h:HashMap<&str, i32> = HashMap::new();
    h.insert("one", 1);
    h.insert("two", 2);
    h.insert("three", 3);
    println!("{:?}", h);

    for (k, v) in &h {
        println!("{}: {}", k, v);
    }
    if let Some(one) = h.get("one") {
        println!("{:?}", one);
    }

    println!("{:?}", h.contains_key("four"));

    let four = h.entry("four").or_insert(4);
    println!("{:?}", four);

    match h.entry("five") {
        std::collections::hash_map::Entry::Occupied(e) => {
            println!("{:?}", e.get());
        },
        std::collections::hash_map::Entry::Vacant(e) => {
            e.insert(5);
        }
    }
    println!("{:?}", h);

    let text = "hello world hello universe hello multiverse";

    let mut counts = HashMap::new();

    for word in text.split_whitespace() {
        *counts.entry(word).or_insert(0) += 1;
    }
    println!("{:?}", counts);
}

