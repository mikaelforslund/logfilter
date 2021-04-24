
mod tokenizer;
mod grammar;

use tokenizer::{ parse, full_lines, Token };
use std::io::{ self };

extern crate pest;
#[macro_use]
extern crate pest_derive;

fn main() -> Result<(), io::Error> {
    // read lines from stdin, tokenises the words using regexps and finally writes same line to stdout... 
    for line in full_lines(io::stdin().lock()) {
        let line = line?;

        for token in line.split_whitespace() {
            let token:Token = parse(&token.to_string());
            println!("{:?}", token);
        }

        println!("full line: {}", line);
    }

    Ok(())
}