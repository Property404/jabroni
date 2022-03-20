use anyhow::Result;
use jabroni::{Binding, BindingMap, Jabroni, Subroutine, Value as JabroniValue};
use std::{
    fmt::Debug,
    fs,
    io::{self, Write},
    path::PathBuf,
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "jabroni", about = "Jabroni interpreter")]
struct Opt {
    file: Option<PathBuf>,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    let mut jabroni = build_jabroni_interpreter()?;

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

fn build_jabroni_interpreter() -> Result<Jabroni> {
    let mut console = BindingMap::default();
    console.set(
        "log".into(),
        Binding::constant(JabroniValue::Subroutine(Subroutine::new_variadic(
            Box::new(|_: BindingMap, args: &mut [JabroniValue]| {
                for (i, arg) in args.iter().enumerate() {
                    print!("{}", arg);
                    if i != args.len() - 1 {
                        print!(" ");
                    }
                }
                println!();
                Ok(JabroniValue::Null)
            }),
        ))),
    );

    let mut interpreter = Jabroni::new();
    interpreter.define_constant("console", JabroniValue::Object(console))?;
    Ok(interpreter)
}
