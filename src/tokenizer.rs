use chrono::{NaiveDate, Utc};
use lazy_static::lazy_static;
use regex::Regex;
use semver::Version;
use std::io::{self, BufRead};
use std::net::Ipv4Addr;
use std::net::Ipv6Addr;

/// Semfilter will be able to handle these types.
///
#[derive(Debug, PartialEq, PartialOrd)]
pub enum Token {
    /// The implicit parameters indicate the following:
    ///   1. symbolic name (e.g. date, string, email etc)
    ///   2. the actual value from the source data
    ///   3. where applicable a format speficier (e.g. dates)
    String(String, String),
    Number(String, f64),
    Integer(String, u64),
    Email(String, String),
    Date(String, NaiveDate, String),
    Ipv4(String, Ipv4Addr),
    Ipv6(String, Ipv6Addr),
    SemVersion(String, Version),
}

impl Token {
    pub fn new_no_validation(
        type_term: &str,
        value: &str,
        format: Option<&str>,
    ) -> Result<Token, String> {
        Token::new(type_term, value, format, false)
    }

    pub fn new(
        type_term: &str,
        value: &str,
        format: Option<&str>,
        validate: bool,
    ) -> Result<Token, String> {
        match type_term {
            "date" => match format {
                Some(format) => Token::try_parse_date(value, format),
                None => Token::try_parse_date(value, "%Y-%m-%d"),
            },

            "email" if validate && EMAIL_REGEX.is_match(value) => Ok(Token::Email(
                String::from(type_term),
                String::from(value),
            )),
            "ivp4" if validate && IPV4_REGEX.is_match(value) => Ok(Token::Ipv4(
                String::from(type_term),
                value.parse().unwrap(),
            )),
            "ivp6" if validate && IPV6_REGEX.is_match(value) => Ok(Token::Ipv6(
                String::from(type_term),
                value.parse().unwrap(),
            )),
            "semver" if validate && SEMVER_REGEX.is_match(value) => Ok(Token::SemVersion(
                String::from(type_term),
                Version::parse(value).unwrap(),
            )),
            "number" if validate && NUMBER_REGEX.is_match(value) => Ok(Token::Number(
                String::from(type_term),
                value.parse().unwrap(),
            )),
            "integer" if validate && INTEGER_REGEX.is_match(value) => Ok(Token::Integer(
                String::from(type_term),
                value.parse().unwrap(),
            )),
            "string" => Ok(Token::String(
                String::from(type_term),
                String::from(value),
            )),

            _ => Err(format!("Type {} not supported", type_term)),
        }
    }

    /// Implements a copy factory method for a Token, currently only tyhe date token is using the format  
    pub fn copy(&self, value: &str, format: Option<&str>) -> Result<Token, String> {
        // override the default format if needed...
        let format = if format.is_some() {
            format
        } else {
            self.get_format()
        };
        Token::new(self.get_type().as_str(), value, format, false)
    }

    fn try_parse_date(value: &str, f: &str) -> Result<Token, String> {
        if value == "now()" {
            Ok(Token::Date(
                String::from("date"),
                Utc::now().naive_utc().date(),
                f.to_string(),
            ))
        } else {
            let date_value = NaiveDate::parse_from_str(value, f);
            if date_value.is_err() {
                return Err(format!(
                    "Problem parsing date value {} using format '{}'",
                    value, f
                ));
            }

            Ok(Token::Date(
                String::from("date"),
                date_value.unwrap(),
                String::from(f),
            ))
        }
    }

    fn get_value_tuple(&self) -> (String, String, Option<&str>) {
        match &*self {
            Token::Date(t, v, f) => (t.to_string(), v.to_string(), Some(f)),
            Token::String(t, v) => (t.to_string(), v.to_string(), None),
            Token::Integer(t, v) => (t.to_string(), v.to_string(), None),
            Token::Number(t, v) => (t.to_string(), v.to_string(), None),
            Token::Email(t, v) => (t.to_string(), v.to_string(), None),
            Token::Ipv4(t, v) => (t.to_string(), v.to_string(), None),
            Token::Ipv6(t, v) => (t.to_string(), v.to_string(), None),
            Token::SemVersion(t, v) => (t.to_string(), v.to_string(), None),
        }
    }

    pub fn get_format(&self) -> Option<&str> {
        let (_, _, format) = self.get_value_tuple();
        format
    }

    pub fn get_value(&self) -> String {
        let (_, value, _) = self.get_value_tuple();
        value
    }

    pub fn get_type(&self) -> String {
        let (token_type, _, _) = self.get_value_tuple();
        token_type
    }

    pub fn is_match(&self, regex_val: &str) -> bool {
        let v = self.get_value();
        return Regex::new(regex_val).unwrap().is_match(v.as_str());
    }
}

// define default type validations...
lazy_static! {
    static ref DATE_REGEX: Regex = Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap();
    static ref EMAIL_REGEX: Regex = Regex::new(r"^\S+@\S+\.\S+$").unwrap();
    static ref IPV4_REGEX: Regex = Regex::new(r"^(?:[0-9]{1,3}\.){3}[0-9]{1,3}$").unwrap();
    static ref IPV6_REGEX: Regex =
        Regex::new(r"^(([0-9a-fA-F]{0,4}:){1,7}[0-9a-fA-F]{0,4})$").unwrap();
    static ref NUMBER_REGEX: Regex = Regex::new(r"^\d+\.(\d{1,2})+$").unwrap();
    static ref INTEGER_REGEX: Regex = Regex::new(r"^\d+$").unwrap();
    static ref SEMVER_REGEX: Regex =
        Regex::new(r"^(0|\d*)\.(0|\d*)\.(0|\d*)(\-\w+(\.\w+)*)?(\+\w+(\.\w+)*)?$").unwrap();
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
    use crate::tokenizer::Token;
    use std::io::Write;

    fn init() {
        let _ = env_logger::builder()
            .format(|buf, record| writeln!(buf, "{}", record.args()))
            .is_test(true)
            .try_init();
    }

    #[test]
    fn test_parse_token() {
        init();

        assert!(matches!(
            Token::new("string", "test", None, true).unwrap(),
            Token::String(_, _),
        ));
        assert!(matches!(
            Token::new("number", "3.14", None, true).unwrap(),
            Token::Number(_, _)
        ));
        assert!(matches!(
            Token::new("integer", "10", None, true).unwrap(),
            Token::Integer(_, _)
        ));
        assert!(matches!(
            Token::new("email", "test@gmail.com", None, true).unwrap(),
            Token::Email(_, _)
        ));
        assert!(matches!(
            Token::new("ivp4", "127.0.0.1", None, true).unwrap(),
            Token::Ipv4(_, _)
        ));
        assert!(matches!(
            Token::new("ivp6", "1762:0:0:0:0:B03:1:AF18", None, true).unwrap(),
            Token::Ipv6(_, _)
        ));
        assert!(matches!(
            Token::new("date", "1970-07-31", None, true).unwrap(),
            Token::Date(_, _, _)
        ));
        assert!(matches!(
            Token::new("date", "1970/07/31", Some("%Y/%m/%d"), true).unwrap(),
            Token::Date(_, _, _)
        ));
        assert!(matches!(
            Token::new("semver", "1.0.0", None, true).unwrap(),
            Token::SemVersion(_, _)
        ));
    }

    #[test]
    fn test_copy() {
        let t = Token::new("string", "string_value", None, true);
        let copy_t = t.unwrap().copy("new_value", None).unwrap();

        assert!(copy_t.get_value() == "new_value");
        assert!(copy_t.get_type() == "string");
    }

    #[test]
    fn test_parse_invalid_format_specifier() {
        assert!(Token::new("date", "1970/07/31", Some("%Y-%m-%d"), true).is_err());
    }

    #[test]
    fn test_is_match() {
        assert!(Token::new("string", "test", None, true)
            .unwrap()
            .is_match("test"));
        assert!(!Token::new("string", "test", None, true)
            .unwrap()
            .is_match("blaha"));
    }

    #[test]
    fn test_parse_invalid_token() {
        assert!(Token::new("invalid", "1.0.0", None, true).is_err());
    }
}
