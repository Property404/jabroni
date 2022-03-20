use anyhow::Result;
use jabroni::{Binding, BindingMap, Jabroni, Subroutine, Value as JabroniValue};
use rustyline::{error::ReadlineError, Editor};
use std::{fmt::Debug, fs, path::PathBuf};
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
        let mut rl = Editor::<()>::new();
        loop {
            match rl.readline("Jabroni> ") {
                Ok(line) => {
                    rl.add_history_entry(line.as_str());
                    match jabroni.run_expression(line.trim()) {
                        Ok(value) => println!("{}", value),
                        Err(e) => println!("{}", e),
                    };
                }
                Err(ReadlineError::Interrupted) => {
                    println!("<Ctrl-C>");
                    break;
                }
                Err(ReadlineError::Eof) => {
                    println!("<Ctrl-D>");
                    break;
                }
                Err(err) => {
                    println!("Error: {:?}", err);
                    break;
                }
            }
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
