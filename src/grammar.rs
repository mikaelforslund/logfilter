

use pest::prec_climber::{Assoc, Operator, PrecClimber};
use pest::Parser;
use pest::iterators::Pair;
use lazy_static::lazy_static;
use crate::tokenizer::{Token};
use log::{trace};

#[derive(Parser)]
#[grammar = "pest_grammar.pest"]
struct SemFilterParser;

/// Public entry function to parse an expression using a list of tokens from the input.  
/// 
/// # Examples
/// ```
/// let tokens: Vec<Token> = vec![create_token("1970-07-31")];
/// 
/// parse_expression("date(1) in [1970-07-31, now()]", &tokens)
/// ```
pub fn parse_expression(expr: &str, tokens: &Vec<Token>) -> bool {
    let mut grammar = SemFilterParser::parse(Rule::grammar, &expr)
        .unwrap_or_else(|e| panic!("{}", e));

    process_pair(expr, grammar.next().unwrap(), &mut Vec::new(), tokens)
}

/// Evaluates two tokens based on its infix operator and returns a bool. Supported operators 
/// are defined in the grammar file. 
/// 
fn eval_op(op: Rule, value: Pair<Rule>, token: &Token) -> bool {
    match op {
        Rule::eq => token == &token.new(value.as_str()),
        Rule::neq => token != &token.new(value.as_str()),
        Rule::lt => return token < &token.new(value.as_str()),
        Rule::gt => return token > &token.new(value.as_str()),
        Rule::lte => token <= &token.new(value.as_str()),
        Rule::gte => token >= &token.new(value.as_str()),
        
        // TODO
        //match_op => {},
        Rule::in_op =>  {             
            trace!("value: {:?}", value);
            let tokens:Vec<Token> = value.into_inner().map(|rule| token.new(rule.as_str())).collect();
            trace!("tokens: {:?}", tokens);
            return tokens.contains(&token);
        },
        _ => false 
    }
}

/// Evaluates a tokenized string expression against a set of rules derived from the semfile grammar 
/// [pest_grammar.pest](pest_grammar.pest)
/// 
fn eval(expr: &str, stack: &mut Vec<Pair<Rule>>, rule: Rule, tokens: &Vec<Token>) -> bool {
    trace!("eval.stack: {:?}", stack);

    let value = stack.pop().unwrap();           // simple value or comma separated value string....
    let op = stack.pop().unwrap();
    let type_term_arg = stack.pop().unwrap();   // n in type(n)
    let type_term = stack.pop().unwrap();       // date, time, timestamp, email, ... 

    // find n'th (type_term_arg) typeTerm among the token whose type == type_term..
    trace!("type_term.as_str {:?}", type_term.as_str());

    let valid_tokens:Vec<&Token> = tokens.into_iter()
        .filter(|&token| token.is_type(type_term.as_str()))
        .collect();

    trace!("evail.token found {:?}", valid_tokens);    
    let n = type_term_arg.as_str().parse::<usize>().unwrap();
    trace!("eval.type_term_arg {:?}", n);

    if valid_tokens.len() < n {
        println!("Invalid type index {}({}) in expression '{}' ({} tokens found, 0-index)", 
            type_term.as_str(), n, expr, valid_tokens.len());
    } else {
        if !valid_tokens.is_empty() {
            match rule {
                Rule::simple_expr => return eval_op(op.as_rule(), value, valid_tokens[n]),
                Rule::contains_expr => return eval_op(op.as_rule(), value, valid_tokens[n]),
                _ => unreachable!("Unexpected rule matched!")
            }
        }
    }
    
    false
}

lazy_static! {
    /// Initializes the PrecClimber which is required for the operator precedence configuration.  
    static ref CLIMBER: PrecClimber<Rule> = {
        PrecClimber::new(vec![
            Operator::new(Rule::and_op, Assoc::Left) | Operator::new(Rule::or_op, Assoc::Left),
        ])    
    };
}

/// Internal function that processes a pest grammar pair and evaluates to true or false. 
/// 
/// * `expr` - The original expression from which the tokens are genered
/// * `pair` - One grammar pair
/// * `stack` - An expression evaluation stack, LIFO
/// * `tokens` - A list of tokens to evaluate    
/// 
fn process_pair<'a>(expr: &str, pair: Pair<'a, Rule>, stack: &mut Vec<Pair<'a, Rule>>, tokens: &Vec<Token>) -> bool {
   
    let atom = |pair| process_pair(expr, pair, stack, tokens);

    let infix = |lhs, op: Pair<Rule>, rhs| match op.as_rule() {
        Rule::and_op => {
            trace!("andOp: lhs: {}, rhs: {}", lhs, rhs);
            return lhs && rhs;
        },
        Rule::or_op => { 
            trace!("orOp: lhs: {}, rhs: {}", lhs, rhs);
            return lhs || rhs;
        },
        _ => unreachable!(),
    };

    let inner_rule = pair.clone();
    trace!("{:?} {:?}", inner_rule.as_rule(),  inner_rule.as_str());

    match pair.as_rule() {
        Rule::expr => {  return CLIMBER.climb(pair.into_inner(), atom, infix);  },
        Rule::simple_expr => { 
            pair.into_inner().map(atom).count();
            return eval(expr, stack, Rule::simple_expr, tokens);
        },
        Rule::contains_expr => { 
            pair.into_inner().map(atom).count();
            return eval(expr, stack, Rule::simple_expr, tokens);
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

    false
}

#[cfg(test)]
mod tests {
    use crate::{create_token, parse_expression, Token};
    use std::io::Write;

    fn init() {
        let _ = env_logger::builder()
            .format(|buf, record| writeln!(buf, "{}", record.args()))
            .is_test(true).try_init();
    }

    #[test]
    fn test_error_reporting() {    
        init();

        let tokens: Vec<Token> = vec![create_token("1970-07-31"), 
            create_token("1900-01-01"), 
            create_token("42"),
            create_token("test")];

        //assert!(parse_expression("date(0) == 1970-07-31", &tokens));
        //assert!(parse_expression("date(1) == 1900-01-01", &tokens));

        // should be true for all tokens...
        assert!( !parse_expression("date(*) == 1900-01-01", &tokens));

        // should fail
        //assert!( !parse_expression("date(9) == 1900-01-01", &tokens));
    }

    #[test]
    fn test_parsing_should_pass() {
        init();

        let tokens: Vec<Token> = vec![create_token("1970-07-31")];

        assert!(parse_expression("date(0) in [1970-07-31, now()]", &tokens));
        assert!(parse_expression("date(0) == 1970-07-31 && date(0) == 1970-07-31 || date(0) == 1970-07-31", &tokens));
        assert!(parse_expression("date(0) == 1970-07-31 && date(0) == 1970-07-31 || date(0) == 1970-07-30", &tokens));

        // these are negative tests....
        assert!( !parse_expression("date(0) == 1970-07-31 && date(0) == 1970-07-30 || date(0) == 1970-07-30", &tokens));
        assert!( !parse_expression("date(0) == 1900-01-01", &tokens));
    }
}
