use anyhow::Result;
use jabroni::Jabroni;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "jabroni", about = "Jabroni interpreter")]
struct Opt {
    file: Option<PathBuf>,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    let mut jabroni = Jabroni::new();
    if let Some(file) = opt.file {
        jabroni.run_script(&fs::read_to_string(file)?)?;
    } else {
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
    Ok(())
}
