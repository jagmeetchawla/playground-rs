fn main() {
    println!("Running traits playground");
    trait Describe {
        fn describe(&self) -> String;
    }

    struct Person {
        first_name: String,
        last_name: String,
    }
    impl Describe for Person {
        fn describe(&self) -> String {
            format!("{} {}", self.first_name, self.last_name)
        }
    }

    let person = Person {
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
    };

    println!("Person name: {}", person.describe());

}