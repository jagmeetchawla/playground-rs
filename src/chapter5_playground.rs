pub fn run() {
    println!("Chapter 5 playground");

    struct Rectangle(i32, i32);
    let rectangle = Rectangle(30, 50);
    println!("The area of rectangle is {}", rectangle.0 * rectangle.1);

    #[derive(Copy, Clone, Debug)]
    struct Point {
        x: i32,
        y: i32,
        z: i32
    }

    #[derive(Copy, Clone, Debug)]
    struct Cuboid {
        p1: Point,
        p2: Point,
    }
    impl Cuboid {
        fn volume(&self) -> i32 {
            let dx = self.p2.x - self.p1.x;
            let dy = self.p2.y - self.p1.y;
            let dz = self.p2.z - self.p1.z;
            dx * dy * dz
        }
        fn surface_area(&self) -> i32 {
            let dx = self.p2.x - self.p1.x;
            let dy = self.p2.y - self.p1.y;
            let dz = self.p2.z - self.p1.z;
            let xy = dx * dy;
            let xz = dx * dz;
            let yz = dy * dz;
            2 * (xy + yz + xz)
        }
    }

    let p1 = Point { x: 0, y: 0, z: 0 };
    let p2 = Point { x: 10, y: 5, z: 2 };

    let cuboid = Cuboid { p1, p2 };
    println!("{:?}", cuboid);

    println!("The volume of cuboid is {}", cuboid.volume());
    println!("The surface area of cuboid is {}", cuboid.surface_area());

    let another_cuboid = cuboid;
    println!("first cuboid: {:?} at {:p}", cuboid, &cuboid);
    println!("second cuboid: {:?} at {:p}", another_cuboid, &another_cuboid);

}