
use std::io::{self, BufRead};
use regex::Regex;
use lazy_static::lazy_static;
use chrono::{NaiveDate, Utc};

/// Semfilter will be able to handle these types.
/// 
#[derive(Debug, PartialEq, PartialOrd)]
pub enum Token {
    /// The implicit parameters indicate the following:
    ///   1. symbolic name (e.g. date, string, email etc) 
    ///   2. the actual value from the source data
    ///   3. where applicable a format speficier (e.g. dates) 
    StringToken(String, String),    
    NumberToken(String, f64),
    IntegerToken(String, u64),
    EmailToken(String, String),
    DateToken(String, NaiveDate, String), 
    Ipv4Token(String, String),
    Ipv6Token(String, String)
}

impl Token {
    /// Implements a copy factory method for a Token 
    pub fn new(&self, value: &str) -> Token {
        match &*self {            
            Token::DateToken(t, _, f) => { 
                if value == "now()".to_string() {
                    Token::DateToken(String::from(t), Utc::now().naive_utc().date(), f.to_string())
                } else {
                    return Token::DateToken(String::from(t), NaiveDate::parse_from_str(value, f).unwrap(), f.to_string());
                }
            },
            Token::StringToken(t, _) =>  Token::StringToken(String::from(t), value.to_string()),
            Token::IntegerToken(t, _) => Token::IntegerToken(String::from(t), value.parse().unwrap()),
            Token::NumberToken(t, _) => Token::NumberToken(String::from(t), value.parse().unwrap()),
            Token::EmailToken(t, _) => Token::EmailToken(String::from(t), value.to_string()),
            Token::Ipv4Token(t, _) => Token::Ipv4Token(String::from(t), value.to_string()),
            Token::Ipv6Token(t, _) => Token::Ipv6Token(String::from(t), value.to_string())
        }
    }

    fn get_value_tuple(&self) -> (String, String) {
        match &*self {
            Token::DateToken(t, s, f) => (t.to_string(), s.format(f.as_str()).to_string()),
            Token::EmailToken(t, s) => (t.to_string(), s.to_string()),
            Token::NumberToken(t, s) => (t.to_string(), s.to_string()),
            Token::IntegerToken(t, s) => (t.to_string(), s.to_string()),
            Token::Ipv4Token(t, s) => (t.to_string(), s.to_string()),
            Token::Ipv6Token(t, s) => (t.to_string(), s.to_string()),
            Token::StringToken(t, s) => (t.to_string(), s.to_string()),
        }
    }

    /// Checks if a token is of a particular type 
    pub fn is_type(&self, str: &str) -> bool {        
        let (t, _) = self.get_value_tuple();
        t == str
    }

    pub fn is_match(&self, regex_val: &str) -> bool {
        //let v = self.to_string();
        let (_, v) = self.get_value_tuple();
        //println!("v: {:?}", v);
        return Regex::new(regex_val).unwrap().is_match(v.as_str());
    }
}

// TODO do we really need to record the format in the Token?
// TODO make the format an additional argument to this function...
pub fn create_token(str: &str) -> Token {
    match str {
        str if DATE_REGEX.is_match(str) => Token::DateToken(String::from("date"), NaiveDate::parse_from_str(str, "%Y-%m-%d").unwrap(), "%Y-%m-%d".to_string()),
        str if EMAIL_REGEX.is_match(str) => Token::EmailToken(String::from("email"), String::from(str)),
        str if IPV4_REGEX.is_match(str) => Token::Ipv4Token(String::from("ipv4"), String::from(str)),
        str if IPV6_REGEX.is_match(str) => Token::Ipv6Token(String::from("ipv6"), String::from(str)),
        str if NUMBER_REGEX.is_match(str) => Token::NumberToken(String::from("number"), str.parse().unwrap()),
        str if INTEGER_REGEX.is_match(str) => Token::IntegerToken(String::from("integer"), str.parse().unwrap()),
        _ => Token::StringToken(String::from("string"), String::from(str))
    }
}

// TODO add more types here... add them here once
lazy_static! {
    static ref DATE_REGEX: Regex = Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap();
    static ref EMAIL_REGEX: Regex = Regex::new(r"^\S+@\S+\.\S+$").unwrap();
    static ref IPV4_REGEX: Regex = Regex::new(r"^(?:[0-9]{1,3}\.){3}[0-9]{1,3}$").unwrap();
    static ref IPV6_REGEX: Regex = Regex::new(r"^(([0-9a-fA-F]{0,4}:){1,7}[0-9a-fA-F]{0,4})$").unwrap();
    static ref NUMBER_REGEX: Regex = Regex::new(r"^\d+(\.\d{1,2})+$").unwrap();
    static ref INTEGER_REGEX: Regex =  Regex::new(r"^\d+$").unwrap();
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
#[cfg(test)]
mod tests {
    use crate::{create_token, Token};
    use std::io::Write;

    fn init() {
        let _ = env_logger::builder()
            .format(|buf, record| writeln!(buf, "{}", record.args()))
            .is_test(true).try_init();
    }

    #[test]
    fn test_parse_token() {       
        init();

        assert!(matches!(create_token("test"), Token::StringToken(_, _)));
        assert!(matches!(create_token("3.14"), Token::NumberToken(_, _)));
        assert!(matches!(create_token("10"), Token::IntegerToken(_, _)));
        assert!(matches!(create_token("test@gmail.com"), Token::EmailToken(_, _)));
        assert!(matches!(create_token("127.0.0.1"), Token::Ipv4Token(_, _)));
        assert!(matches!(create_token("1762:0:0:0:0:B03:1:AF18"), Token::Ipv6Token(_, _)));
        assert!(matches!(create_token("1970-07-31"), Token::DateToken(_, _, _)));
    }

    #[test]
    fn test_token_is_match() {       
        init();

        assert!(create_token("test").is_type("string"));
        assert!(create_token("3.14").is_type("number"));
        assert!(create_token("10").is_type("integer"));
        assert!(create_token("test@gmail.com").is_type("email"));
        assert!(create_token("127.0.0.1").is_type("ipv4"));
        assert!(create_token("1762:0:0:0:0:B03:1:AF18").is_type("ipv6"));
        assert!(create_token("1970-07-31").is_type("date"));

        assert!( !create_token("test").is_type("s"));
        assert!( !create_token("3.14").is_type("n"));
        assert!( !create_token("10").is_type("i"));
        assert!( !create_token("test@gmail.com").is_type("e"));
        assert!( !create_token("127.0.0.1").is_type("i"));
        assert!( !create_token("1762:0:0:0:0:B03:1:AF18").is_type("i"));
        assert!( !create_token("1970-07-31").is_type("d"));
    }
}
