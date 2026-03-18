pub fn run() {
    fn pair<T, U>(x: T, y: U) -> (T, U) {
        (x, y)
    }

    let p = pair(1, 2);
    println!("{:?}", p);

    let p = pair("a", "b");
    println!("{:?}", p);

    let p = pair(10, "twenty");
    println!("{:?}", p);

    #[derive(Debug)]
    struct Point<T> {
        x: T,
        y: T,
    }
    impl Point<f64> {
        fn distance_from_origin(&self) -> f64 {
            (self.x * self.x + self.y * self.y).sqrt()
        }
    }

    let p = Point { x: 2.0, y: 1.0 };
    println!("{:?}", p);
    println!("Distance from origin: {:?}", p.distance_from_origin());

    fn first<T: std::fmt::Debug>(items: Vec<T>) -> Option<T> {
        items.into_iter().next()
    }

    let v = vec![1, 2, 3];
    let first = first(v);
    println!("{:?}", first);

}