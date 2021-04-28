
mod tokenizer;
mod grammar;

use tokenizer::{ create_token, full_lines, Token };
use grammar:: { parse_expression };
use std::io::{ self, Write };
use log::{info};
use simple_logger::SimpleLogger;
use log::LevelFilter;
use std::{env, process};

extern crate pest;
#[macro_use]
extern crate pest_derive;

fn main() -> Result<(), io::Error> {
    SimpleLogger::new().with_level(LevelFilter::Trace).init().unwrap();

    /////////////////// TODO move this to clap... this is just for testing... 
    let args: Vec<String> = env::args().collect();   

    if args.len() == 1 {
        info!("\n\nUsage: semfilter [expr] < FILE\n");
        process::exit(-1);
    }
    info!("arg: {:?}", args[1]);
    ////////////////////

    // read lines from stdin, tokenizes the words using regexps and finally writes same line to stdout if it 
    // matches the expression passed in to the program 
    for line in full_lines(io::stdin().lock()) {
        let line = line?;

        let tokens:Vec<Token> = line.split_whitespace()
            .into_iter()
            .map(|word| create_token(&word))
            .collect();

        info!("main.tokens: {:?}", tokens);

        if parse_expression(&args[1], &tokens) {
            let stdout = io::stdout();
            let mut handle = stdout.lock();
            handle.write_all(line.as_bytes())?;
        }
    }

    Ok(())
}