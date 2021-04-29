
mod tokenizer;
mod grammar;

use tokenizer::{ create_token, full_lines, Token };
use grammar:: { parse_expression };
use std::io::{ self, Write };
use log::{trace};
use std::{env, process};

extern crate pest;
#[macro_use]
extern crate pest_derive;

/// The main program loop which basically reads lines from stdin, tokenizes the words and runs them through the 
/// parsed expression and if evaluates to true, writes the same line to stdout. Logging can be configured by 
/// setting the envvariable RUST_LOG to any in [trace, info, warn, error] as described here 
/// [env_logger](https://crates.io/crates/env_logger) 
/// 
fn main() -> Result<(), io::Error> {
    env_logger::init();

    /////////////////// TODO move this to clap... this is just for testing... 
    let args: Vec<String> = env::args().collect();   

    if args.len() == 1 {
        println!("\n\nUsage: semfilter [expr] < FILE\n");
        process::exit(-1);
    }
    trace!("arg: {:?}", args[1]);
    ////////////////////

    // read lines from stdin, tokenizes the words using regexps and finally writes same line to stdout if it 
    // matches the expression passed in to the program 
    for line in full_lines(io::stdin().lock()) {
        let line = line?;

        let tokens:Vec<Token> = line.split_whitespace()
            .into_iter()
            .map(|word| create_token(&word))
            .collect();

        trace!("main.tokens: {:?}", tokens);

        if parse_expression(&args[1], &tokens) {
            let stdout = io::stdout();
            let mut handle = stdout.lock();
            handle.write_all(line.as_bytes())?;
        }
    }

    Ok(())
}