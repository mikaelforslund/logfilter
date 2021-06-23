

use pest::prec_climber::{Assoc, Operator, PrecClimber};
use pest::Parser;
use pest::iterators::{Pair, Pairs};
use lazy_static::lazy_static;
use crate::tokenizer::{Token, create_token};
use log::{trace};

#[derive(Parser)]
#[grammar = "pest_grammar.pest"]
struct SemFilterParser;


lazy_static! {
    /// Initializes the PrecClimber which is required for the operator precedence configuration.  
    static ref CLIMBER: PrecClimber<Rule> = {
        PrecClimber::new(vec![
            Operator::new(Rule::and_op, Assoc::Left) | Operator::new(Rule::or_op, Assoc::Left),
        ])    
    };
}

/// Public entry function to parse an expression using a list of tokens from the input.  
/// 
/// # Examples
/// ```
/// let tokens: Vec<Token> = vec![create_token("1970-07-31")];
/// 
/// parse_expression("date(1) in [1970-07-31, now()]", &tokens)
/// ```
pub fn parse_expression<'a>(expr: &'a str) -> Result<Pairs<'a, Rule>, String> {
    match SemFilterParser::parse(Rule::grammar, &expr) {
      Ok(grammar) => Ok(grammar),
      Err(e) => Err(e.to_string()),      
    }
}

pub fn evaluate_line(grammar: &mut Pairs<Rule>, tokens: &Vec<&str>) -> Result<bool, String> {
    match process_grammar(grammar.next().unwrap(), &mut Vec::new(), tokens) {
        Ok(val) => Ok(val),
        Err(e) => Err(e.to_string())
    } 
}

/// Evaluates two tokens based on its infix operator and returns Result. Supported operators 
/// are defined in the grammar file. 
/// 
fn eval_op(type_term: &str, op: Rule, value: Pair<Rule>, token_val: &str) -> Result<bool, String> {
    trace!("op {:?}, value: {:?}, token {:?}", op, value, token_val);

    match create_token(type_term, token_val) {
        Ok(token) => {
            match op {
                Rule::eq => Ok(token == token.new(value.as_str())),
                Rule::neq => Ok(token != token.new(value.as_str())),
                Rule::lt => Ok(token < token.new(value.as_str())),
                Rule::gt => Ok(token > token.new(value.as_str())),
                Rule::lte => Ok(token <= token.new(value.as_str())),
                Rule::gte => Ok(token >= token.new(value.as_str())),
                        
                Rule::match_op => {
                    //println!("the token: {:?}", token);
                    match &token {
                        Token::StringToken(_, _) => Ok(token.is_match(value.as_str())),
                        _ => Err(format!("Invalid token type {}:{}, only string type is allowed for match expressions", 
                                type_term, token_val))
                    } 
                },
                Rule::in_op =>  {             
                    let tokens:Vec<Token> = value.into_inner().map(|rule| token.new(rule.as_str())).collect();
                    return Ok(tokens.contains(&token));
                },
                _ => Ok(false) 
            }       
        },
        Err(_) => Ok(false)
    }
}

/// Evaluates a tokenized string expression against a set of rules derived from the semfile grammar 
/// [pest_grammar.pest](pest_grammar.pest)
/// 
fn eval(stack: &mut Vec<Pair<Rule>>, rule: Rule, tokens: &Vec<&str>) -> Result<bool, String> {
    trace!("eval.stack: {:?}", stack);

    let value = stack.pop().unwrap();           // simple value or comma separated value string....
    let op = stack.pop().unwrap();
    let type_term_arg = stack.pop().unwrap();   // n in type(n)
    let type_term = stack.pop().unwrap();       // date, time, timestamp, email, ... 

    // find n'th (type_term_arg) typeTerm among the token whose type == type_term..
    trace!("type_term.as_str {:?}", type_term.as_str());

    match type_term_arg.as_str() {
        "*" => {
            // eval all tokens that matches the type_term, e.g: 
            //   true for: date(*) == 1900-01-01    for tokens:[1900-01-01, 1900-01-01]
            //   false for: date(*) == 1900-01-01   for tokens:[1970-07-31, 1900-01-01]
            return Ok(tokens
                    .into_iter()
                    .all(|t| eval_op(type_term.as_str(), op.as_rule(), value.clone(), t).unwrap()));
        },
        index => {
            let n = index.parse::<usize>().unwrap();

            if tokens.len()-1 < n {
                return Ok(false);
            }          
            
            match rule {                
                // eval the token that matched  the type_term, e.g: 
                //   true for: date(1) == 1900-01-01    for tokens:[1900-01-01, 1970-07-31]
                //   false for: date(2) == 1900-01-01   for tokens:[1900-01-01, 1970-07-31]
                Rule::simple_expr => eval_op(type_term.as_str(), op.as_rule(), value, tokens[n]), 
                Rule::contains_expr => return eval_op(type_term.as_str(), op.as_rule(), value, tokens[n]),
                _ => return Err(String::from("Unexpected rule matched!")),
            } 
        }
    } 
}

/// Internal function that processes a pest grammar pair and evaluates to Result. 
/// 
/// * `pair` - One grammar pair
/// * `stack` - An expression evaluation stack, LIFO
/// * `tokens` - A list of tokens to evaluate    
/// 
fn process_grammar<'a>(pair: Pair<'a, Rule>, stack: &mut Vec<Pair<'a, Rule>>, tokens: &Vec<&str>) 
    -> Result<bool, String> {
   
    let atom = |pair| process_grammar(pair, stack, tokens);

    let infix = |lhs: Result<bool, String>, op: Pair<Rule>, rhs: Result<bool, String>| 
        -> Result<bool, String> {

        match op.as_rule() {
            Rule::and_op => {
                trace!("andOp: lhs: {:?}, rhs: {:?}", lhs, rhs);
                Ok(lhs? && rhs?)
            },
            Rule::or_op => { 
                trace!("orOp: lhs: {:?}, rhs: {:?}", lhs, rhs);
                Ok(lhs? || rhs?)
            },
            _ => Err(String::from("Unexpected rule found!"))
       }
    };

    let inner_rule = pair.clone();
    trace!("{:?} {:?}", inner_rule.as_rule(),  inner_rule.as_str());

    match pair.as_rule() {
        Rule::expr => {  return CLIMBER.climb(pair.into_inner(), atom, infix);  },
        Rule::simple_expr => { 
            pair.into_inner().map(atom).count();
            return eval(stack, Rule::simple_expr, tokens);
        },
        Rule::contains_expr => { 
            pair.into_inner().map(atom).count();
            return eval(stack, Rule::simple_expr, tokens);
        },

        Rule::type_expr => { pair.into_inner().map(atom).count(); },
        Rule::type_term => stack.push(pair),
        Rule::type_term_arg => stack.push(pair),
        Rule::op => stack.push(pair.into_inner().next().unwrap()),
        Rule::in_op => stack.push(pair),
        Rule::value => stack.push(pair),
        Rule::list_expr => { pair.into_inner().map(atom).count(); },
        Rule::list_member_expr => stack.push(pair),

        _ => trace!("_: {}", pair.as_str())
    }

    Ok(false)
}

#[cfg(test)]
mod tests {

    // TODO incorporate this patterm to get a @Before behaviour to run before each test...
    // #[test]
    // fn test_something_interesting() {
    //     run_test(|| {
    //         let true_or_false = do_the_test();
    
    //         assert!(true_or_false);
    //     })
    // }
    // fn run_test<T>(test: T) -> ()
    //     where T: FnOnce() -> () + panic::UnwindSafe
    // {
    //     setup();
    
    //     let result = panic::catch_unwind(|| {
    //         test()
    //     });
    
    //     teardown();
    
    //     assert!(result.is_ok())
    // }

    use crate::grammar::parse_expression; 
    use crate::tokenizer::{create_token, Token};
    use std::io::Write;

    fn init() {
        let _ = env_logger::builder()
            .format(|buf, record| writeln!(buf, "{}", record.args()))
            .is_test(true).try_init();
    }

    #[test]
    fn test_empty_tokens() {
        assert!(parse_expression("date(0) == 1970-07-31", &vec!()).is_ok());
    }

    #[test]
    fn test_empty_expressions() {    
        assert!(parse_expression("", &vec!()).is_err());
    }

    #[test]
    fn test_error_reporting() {    
        init();

        let tokens: Vec<Token> = vec![create_token("1970-07-31"), 
            create_token("1900-01-01"), 
            create_token("42"),
            create_token("test")];

        assert!(parse_expression("date(0) == 1970-07-31", &tokens).is_ok());
        assert!(parse_expression("date(1) == 1900-01-01", &tokens).is_ok());

        // should fail
        assert!(parse_expression("date(9) == 1900-01-01", &tokens).unwrap() == false);
    }

    #[test]
    fn test_match() { 
        let tokens: Vec<Token> = vec![create_token("1970-07-31")];

        assert!(parse_expression("date(0) match \\d{4}-\\d{2}-\\d{2}", &tokens).unwrap() == true);
    }


    #[test]
    fn test_multiple_token_eval() {    
        init();

        let tokens: Vec<Token> = vec![create_token("1970-07-31"), create_token("1900-01-01"), create_token("test")];
 
        assert!(parse_expression("date(*) == 1900-01-01", &tokens).unwrap() == false);

        let tokens: Vec<Token> = vec![create_token("1970-07-31"), create_token("1970-07-31"), create_token("test")];
 
        assert!(parse_expression("date(*) == 1970-07-31", &tokens).unwrap() == true);
    }


    #[test]
    fn test_parsing_should_pass() {
        init();

        let tokens: Vec<Token> = vec![create_token("1970-07-31")];

        assert!(parse_expression("date(0) in [1970-07-31, now()]", &tokens).is_ok());
        assert!(parse_expression("date(0) == 1970-07-31 && date(0) == 1970-07-31 || date(0) == 1970-07-31", &tokens).is_ok());
        assert!(parse_expression("date(0) == 1970-07-31 && date(0) == 1970-07-31 || date(0) == 1970-07-30", &tokens).is_ok());

        // these are negative tests....
        assert!(parse_expression("date(0) == 1970-07-31 && date(0) == 1970-07-30 || date(0) == 1970-07-30", &tokens).unwrap() == false);
        assert!(parse_expression("date(0) == 1900-01-01", &tokens).unwrap() == false);
    }
}
