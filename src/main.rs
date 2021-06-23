
mod tokenizer;
mod grammar;
mod cli;
mod processor; 

#[macro_use]
extern crate clap;

extern crate pest;

#[macro_use]
extern crate pest_derive;

fn main() -> Result<(), std::io::Error> {
    env_logger::init();

    processor::process_input(cli::parse_cli())
}