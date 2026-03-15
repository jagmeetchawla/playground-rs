pub fn run() {
    enum Color {
        Red,
        Green,
        Blue
    }

    let _ = Color::Blue;
    let c = Color::Red;
    let _ = Color::Green;

    match c {
        Color::Red => println!("\x1b[31mThis is red\x1b[0m"),
        Color::Green => println!("\x1b[32mThis is green\x1b[0m"),
        Color::Blue => println!("\x1b[34mThis is blue\x1b[0m"),
    }

    enum IpAddress {
        V4([u8; 4]),
        V6([u16; 8])
    }

    impl IpAddress {
        fn is_v4(&self) -> bool {
            if let IpAddress::V4(i) = self {
                println!("This is v4: {}:{}:{}:{}", i[0], i[1], i[2], i[3]);
                return true;
            }
            false
        }

        fn is_v6(&self) -> bool {
            if let IpAddress::V6(i) = self {
                println!("This is v6: {:#x}:{:#x}:{:#x}:{:#x}:{:#x}:{:#x}:{:#x}:{:#x}", i[0], i[1], i[2], i[3], i[4], i[5], i[6], i[7]);
                return true;
            }
            false
        }

    }

    let home = IpAddress::V4([127, 0, 0, 1]);
    let home2 = IpAddress::V6([0, 0, 0, 0, 0, 0, 0, 255]);

    home.is_v4();
    home2.is_v6();

    let val1 = Some(5);
    let val2 = 10;
    let val3: Option<i32> = None;

    let val4 = match val1 {
        Some(v) => Some(v + val2),
        None => None
    };
    println!("{:?}", val1);
    println!("{:?}", val2);
    println!("{:?}", val3);
    println!("{:?}", val4);
    println!("{:?}", val1.unwrap_or(0));
    println!("{:?}", val3.unwrap_or_else(|| 0));

    if let Some(v) = val4 {
        println!("v = {:?}", v);
    }

    if let Some(v) = val3 {
        println!("v = {:?}", v);
    }

}