
mod parser;

use parser::{ parse, full_lines, Token };
use std::io::{ self };

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