use std::io::prelude::*;
use std::fs::OpenOptions;
use std::path::Path;
//use std::fs::File;
fn main() {
    println!("Running chapter 9 playground");

    let filename = "content/hello.txt";

    if Path::new(filename).exists() {
        println!("file exists: {}", filename);
        let contents = std::fs::read_to_string(filename).unwrap_or_else(|e| panic!("Failed to read file: {}", e));
        println!("{}", contents);
    } else {
        println!("file does not exist: {}", filename);
        let mut f = OpenOptions::new()
            .create(true)
            .write(true)
            .open(filename).unwrap_or_else(|e| panic!("Failed to create file: {}", e));
        f.write_all(b"Hello, world!\n").unwrap_or_else(|e| panic!("Failed to write to file: {}", e));
    }

    /*
    let mut file = File::open(filename).unwrap_or_else(
        |e| match e.kind() {
            std::io::ErrorKind::NotFound => {
                let mut f = File::create(filename).unwrap_or_else(|e| panic!("Failed to create file: {}", e));
                f.write_all(b"Hello, world 2!\n").unwrap();
                f.flush().unwrap();
                f
            },
            _ => panic!("Unexpected error: {}", e),
        }
    );
    let mut contents = String::new();
    if let Ok(content_length) = file.read_to_string(&mut contents) {
        println!("Read {} bytes", content_length);
    }
    println!("{}", contents)
    */
    /*
    match file {
        Ok(mut f) => {
            let mut contents = String::new();
            f.read_to_string(&mut contents).unwrap();
            println!("{}", contents);
        },
        Err(e) => {
            println!("Error: {}", e);
            println!("{:?} - {:?}", e.raw_os_error().unwrap(), e.kind());
            match e.kind() {
                std::io::ErrorKind::NotFound => {
                    println!("File not found: {}", filename);
                    let f = File::create(filename);
                    match f {
                        Ok(mut f) => {
                            f.write_all(b"Hello, world 2!").unwrap();
                            println!("Created new file {}", filename);
                        },
                        Err(_e) => {
                            panic!("Failed to create file");
                        }
                    }
                },
                _ => panic!("Unexpected error"),
            }
        },
    }
    */
}
