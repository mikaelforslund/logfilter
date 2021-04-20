
use std::io::{self, BufRead};
use regex::Regex;
use lazy_static::lazy_static;
use std::collections::HashMap;

#[derive(Debug)]
pub enum Token {
    StringToken(String),
    NumberToken(f64),
    IntegerToken(u64),
    EmailToken(String),
    DateToken(String, String), 
    IPv4Token(String, String),
    IPv6Token(String, String)
}

fn is_match(token_type: &TokenType, str: &String) -> bool {
    match TOKENTYPE_MAP.get(&token_type) {
        Some(r) => r.is_match(&str),
        None => false
    }
}

// TODO do we really need to record the format in the Token?
pub fn parse(str: &String) -> Token {
    match str {
        str if is_match(&TokenType::Date, &str) => Token::DateToken(str.to_string(), "dateFormat".to_string()),
        str if is_match(&TokenType::Email, &str) => Token::EmailToken(str.to_string()),
        str if is_match(&TokenType::Ipv4, &str) => Token::IPv4Token(str.to_string(), "ipv4Format".to_string()),
        str if is_match(&TokenType::Ipv6, &str) => Token::IPv6Token(str.to_string(), "ipv6Format".to_string()),
        str if is_match(&TokenType::Number, &str) => Token::NumberToken(str.parse().unwrap()),
        str if is_match(&TokenType::Integer, &str) => Token::IntegerToken(str.parse().unwrap()),
        // TODO add mechanism to be able to add more definitions dynamically?
        _ => Token::StringToken(str.to_string())       
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum TokenType {
    Date, Email, Number, Integer, Ipv4, Ipv6 
}

// TODO add more types here...
lazy_static! {
    static ref TOKENTYPE_MAP: HashMap<TokenType, Regex> = {
        let mut map = HashMap::new();
        map.insert(TokenType::Date, Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap());
        map.insert(TokenType::Email, Regex::new(r"^\S+@\S+\.\S+$").unwrap());
        map.insert(TokenType::Ipv4, Regex::new(r"^(?:[0-9]{1,3}\.){3}[0-9]{1,3}$").unwrap());
        map.insert(TokenType::Ipv6, Regex::new(r"^(([0-9a-fA-F]{0,4}:){1,7}[0-9a-fA-F]{0,4})$").unwrap());
        map.insert(TokenType::Number, Regex::new(r"^\d+(\.\d{1,2})+$").unwrap());
        map.insert(TokenType::Integer, Regex::new(r"^\d+$").unwrap());
        map
    };
}

pub fn full_lines(mut input: impl BufRead) -> impl Iterator<Item = io::Result<String>> {
    std::iter::from_fn(move || {
        let mut vec = String::new();
        match input.read_line(&mut vec) {
            Ok(0) => None,
            Ok(_) => Some(Ok(vec)),
            Err(e) => Some(Err(e)),
        }
    })
}