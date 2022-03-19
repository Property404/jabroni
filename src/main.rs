use jabroni::Jabroni;
use std::io::{self, Write};
fn main() {
    let mut jabroni = Jabroni::new();

    loop {
        print!("Jabroni> ");
        io::stdout().flush().unwrap();

        let mut line = String::new();
        io::stdin()
            .read_line(&mut line)
            .expect("Failed to read line");
        match jabroni.run_expression(line.trim()) {
            Ok(value) => println!("{}", value),
            Err(e) => println!("{}", e),
        };
    }
}
