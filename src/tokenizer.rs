
use std::io::{self, BufRead};
use regex::Regex;
use lazy_static::lazy_static;
use chrono::{NaiveDate, Utc};
use std::net::Ipv4Addr;
use std::net::Ipv6Addr;
use semver::Version;

/// Semfilter will be able to handle these types.
/// 
#[derive(Debug, PartialEq,  PartialOrd)]
pub enum Token {
    /// The implicit parameters indicate the following:
    ///   1. symbolic name (e.g. date, string, email etc) 
    ///   2. the actual value from the source data
    ///   3. where applicable a format speficier (e.g. dates) 
    StringToken(String),    
    NumberToken(f64),
    IntegerToken(u64),
    EmailToken(String),
    DateToken(NaiveDate, String), 
    Ipv4Token(Ipv4Addr),
    Ipv6Token(Ipv6Addr),
    SemVersionToken(Version)
}

impl Token {
    /// Implements a copy factory method for a Token 
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
            Token::IntegerToken(_) => { 
                println!("value: {:?}", value);
                Token::IntegerToken(value.parse().unwrap())
            },
            Token::NumberToken(_) => Token::NumberToken(value.parse().unwrap()),
            Token::EmailToken(_) => Token::EmailToken(value.to_string()),
            Token::Ipv4Token(_) => Token::Ipv4Token(value.parse().unwrap()),
            Token::Ipv6Token(_) => Token::Ipv6Token(value.parse().unwrap()),
            Token::SemVersionToken(_) => Token::SemVersionToken(value.parse().unwrap())
        }
    }

    fn get_value(&self) -> String {
        match &*self {
            Token::DateToken(s, _) => s.to_string(),
            Token::EmailToken(s) => s.to_string(),
            Token::NumberToken(s) => s.to_string(),
            Token::IntegerToken(s) => s.to_string(),
            Token::Ipv4Token(s) => s.to_string(),
            Token::Ipv6Token(s) => s.to_string(),
            Token::StringToken(s) => s.to_string(),
            Token::SemVersionToken(s) => s.to_string()
        }
    }

    pub fn is_match(&self, regex_val: &str) -> bool {
        let v = self.get_value();
        return Regex::new(regex_val).unwrap().is_match(v.as_str());
    }
}

// TODO do we really need to record the format in the Token?
// TODO make the format an additional argument to this function...
pub fn create_token(type_term: &str, value: &str) -> Result<Token, String> {
    match type_term {
        type_term if type_term == "date" && DATE_REGEX.is_match(value) => 
            Ok(Token::DateToken(NaiveDate::parse_from_str(value, "%Y-%m-%d").unwrap(), "%Y-%m-%d".to_string())),

        type_term if type_term == "email" && EMAIL_REGEX.is_match(value) => 
            Ok(Token::EmailToken(String::from(value))),

        type_term if type_term == "ivp4" && IPV4_REGEX.is_match(value) => 
            Ok(Token::Ipv4Token(value.parse().unwrap())),

        type_term if type_term == "ivp6" && IPV6_REGEX.is_match(value) => 
            Ok(Token::Ipv6Token(value.parse().unwrap())),

        type_term if type_term == "semver" && SEMVER_REGEX.is_match(value) => 
            Ok(Token::SemVersionToken(Version::parse(value).unwrap())),

        type_term if type_term == "number" && NUMBER_REGEX.is_match(value) => 
            Ok(Token::NumberToken(value.parse().unwrap())),

        type_term if type_term ==  "integer" && INTEGER_REGEX.is_match(value) => 
            Ok(Token::IntegerToken(value.parse().unwrap())),

        type_term if type_term == "string" => Ok(Token::StringToken(String::from(value))),
        
        _ => Err(String::from(format!("Type {} not supported", type_term)))
    }
}

// define type validations...
lazy_static! {
    static ref DATE_REGEX: Regex = Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap();
    static ref EMAIL_REGEX: Regex = Regex::new(r"^\S+@\S+\.\S+$").unwrap();
    static ref IPV4_REGEX: Regex = Regex::new(r"^(?:[0-9]{1,3}\.){3}[0-9]{1,3}$").unwrap();
    static ref IPV6_REGEX: Regex = Regex::new(r"^(([0-9a-fA-F]{0,4}:){1,7}[0-9a-fA-F]{0,4})$").unwrap();
    static ref NUMBER_REGEX: Regex = Regex::new(r"^\d+\.(\d{1,2})+$").unwrap();
    static ref INTEGER_REGEX: Regex =  Regex::new(r"^\d+$").unwrap();
    static ref SEMVER_REGEX: Regex =  Regex::new(r"^(0|\d*)\.(0|\d*)\.(0|\d*)(\-\w+(\.\w+)*)?(\+\w+(\.\w+)*)?$").unwrap();
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
mod tests {   
    use crate::tokenizer::{create_token, Token};
    use std::io::Write;

    //use crate::{ Tok, EqToken, OrdEqToken };

    //use  crate::{ Tok, EqToken, OrdEqToken };
    
    fn init() {
        let _ = env_logger::builder()
            .format(|buf, record| writeln!(buf, "{}", record.args()))
            .is_test(true).try_init();
    }

    #[test]
    fn test_parse_token() {       
        init();

        assert!(matches!(create_token("string", "test").unwrap(), Token::StringToken(_)));
        assert!(matches!(create_token("number", "3.14").unwrap(), Token::NumberToken(_)));
        assert!(matches!(create_token("integer", "10").unwrap(), Token::IntegerToken(_)));
        assert!(matches!(create_token("email", "test@gmail.com").unwrap(), Token::EmailToken(_)));
        assert!(matches!(create_token("ivp4", "127.0.0.1").unwrap(), Token::Ipv4Token(_)));
        assert!(matches!(create_token("ivp6","1762:0:0:0:0:B03:1:AF18").unwrap(), Token::Ipv6Token(_)));
        assert!(matches!(create_token("date", "1970-07-31").unwrap(), Token::DateToken(_, _)));
        assert!(matches!(create_token("semver","1.0.0").unwrap(), Token::SemVersionToken(_)));
    }
}
