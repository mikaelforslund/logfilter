
use std::io::{self, BufRead};
use regex::Regex;
use lazy_static::lazy_static;
use std::collections::HashMap;
use chrono::{NaiveDate, Utc};

#[derive(Debug, PartialEq, PartialOrd)]
pub enum Token {
    StringToken(String),
    NumberToken(f64),
    IntegerToken(u64),
    EmailToken(String),
    DateToken(NaiveDate, String), 
    IPv4Token(String),
    IPv6Token(String)
}

impl Token {
    pub fn new(&self, value: &str) -> Token {
        match &*self {
            Token::DateToken(_, f) => { 
                if value == "now()".to_string() {
                    Token::DateToken(Utc::now().naive_utc().date(), f.to_string())
                } else {
                    return Token::DateToken(NaiveDate::parse_from_str(value, f).unwrap(), f.to_string());
                }
            },
            Token::StringToken(_) =>  Token::StringToken(value.to_string()),
            Token::IntegerToken(_) => Token::IntegerToken(value.parse().unwrap()),
            Token::NumberToken(_) => Token::NumberToken(value.parse().unwrap()),
            Token::EmailToken(_) => Token::EmailToken(value.to_string()),
            Token::IPv4Token(_) => Token::IPv4Token(value.to_string()),
            Token::IPv6Token(_) => Token::IPv6Token(value.to_string())
        }
    }
}

fn is_match(token_type: &TokenType, str: &str) -> bool {
    match TOKENTYPE_MAP.get(&token_type) {
        Some(r) => r.is_match(&str),
        None => false
    }
}

// TODO do we really need to record the format in the Token?
// TODO make the format an additional argument to this function...
pub fn create_token(str: &str) -> Token {
    match str {
        str if is_match(&TokenType::Date, str) => Token::DateToken(NaiveDate::parse_from_str(str, "%Y-%m-%d").unwrap(), "%Y-%m-%d".to_string()),
        str if is_match(&TokenType::Email, str) => Token::EmailToken(str.to_string()),
        str if is_match(&TokenType::Ipv4, str) => Token::IPv4Token(str.to_string()),
        str if is_match(&TokenType::Ipv6, str) => Token::IPv6Token(str.to_string()),
        str if is_match(&TokenType::Number, str) => Token::NumberToken(str.parse().unwrap()),
        str if is_match(&TokenType::Integer, str) => Token::IntegerToken(str.parse().unwrap()),
        
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
        // TODO make the date format more flexible to be able to use parse_from_str() to work with arbitrary formats..
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

#[cfg(test)]
#[test]
fn test_parse_token() {        
    assert!(matches!(create_token("test"), Token::StringToken(_)));
    assert!(matches!(create_token("3.14"), Token::NumberToken(_)));
    assert!(matches!(create_token("10"), Token::IntegerToken(_)));
    assert!(matches!(create_token("test@gmail.com"), Token::EmailToken(_)));
    assert!(matches!(create_token("127.0.0.1"), Token::IPv4Token(_)));
    assert!(matches!(create_token("1762:0:0:0:0:B03:1:AF18"), Token::IPv6Token(_)));
    assert!(matches!(create_token("1970-07-31"), Token::DateToken(_, _)));
}