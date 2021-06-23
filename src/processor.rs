
use crate::tokenizer::{ full_lines};
use crate::grammar::{ parse_expression, evaluate_line };
use std::io::{ self, Write };
use crate::cli::CommandArgs;
use log::{trace};

/// The processor loop which reads lines from stdin, tokenizes the words and runs them through the 
/// parsed expression and if evaluates to true, writes the same line to stdout. Logging can be configured by 
/// setting the envvariable RUST_LOG to any in [trace, info, warn, error] as described here 
/// [env_logger](https://crates.io/crates/env_logger) 
/// 
pub fn process_input(command_args: CommandArgs) -> Result<(), io::Error> {
    match command_args {
        CommandArgs { expr, data_def, token_regex } => {
    
            println!("expr: {:?}  data_def: {:?},  token_sep: {:?}", expr, data_def, token_regex);
    
            match parse_expression(expr.as_str()) {
                Ok(mut grammar) => {

                    // read lines from stdin, tokenizes the words using regexps and finally writes same line to stdout if it 
                    // matches the expression passed in to the program 
                    for line in full_lines(io::stdin().lock()) {
                        let line = line?;                        

                        let matches_regex = |char: char| { 
                            match &token_regex {
                                Some(regex) => { println!("char: {:?}", char);  return regex.is_match(char.to_string().as_str()); },
                                None => char == ' '
                            }
                        };

                        // TODO take the token separator characte set and split on that..
                        let tokens = line.split(matches_regex)
                            .into_iter()
                            .map(|word| word.trim())
                            .collect::<Vec<&str>>();

                        trace!("main.tokens: {:?}", tokens);
            
                        match evaluate_line(&mut grammar, &tokens) {            
                            Ok(true) => {
                                let stdout = io::stdout();
                                let mut handle = stdout.lock();
                                handle.write_all(line.as_bytes())?;
                            },
                            Err(e) => println!("Error in expression: {}", e),
                            _ => {}
                        }
                    }
                },
                Err(e) => /*io::Error::new(ErrorKind::Other, "oh no!") // */ println!("Error parsing epression: {}", e.to_string())
            }
        }
    }

    Ok(())
}