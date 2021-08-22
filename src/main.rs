mod cli;
mod grammar;
mod processor;
mod tokenizer;

#[macro_use]
extern crate clap;

extern crate pest;

#[macro_use]
extern crate pest_derive;

fn main() -> Result<(), std::io::Error> {
    env_logger::init();

    processor::process_input(cli::parse_cli())
}
