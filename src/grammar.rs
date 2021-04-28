

use pest::prec_climber::{Assoc, Operator, PrecClimber};
use pest::Parser;
use pest::iterators::Pair;
use lazy_static::lazy_static;
use crate::tokenizer::{Token};
use log::{trace};

#[derive(Parser)]
#[grammar = "pest_grammar.pest"]
struct SemFilterParser;

pub fn parse_expression(expr: &str, tokens: &Vec<Token>) -> bool {
    let mut grammar = SemFilterParser::parse(Rule::grammar, &expr)
        .unwrap_or_else(|e| panic!("{}", e));

    process_pair(grammar.next().unwrap(), &mut Vec::new(), tokens)
}

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
        _ => {
            return false  
        }
    }
}

/// Evaluates a tokenized string expression against a set of rules derived from the semfile grammar 
/// (pest_grammar.pest)
/// 
/// # Examples
/// 
/// let tokens: Vec<Token> = vec![create_token("1970-07-31")];
/// 
/// parse_expression("date(1) in [1970-07-31, now()]", &tokens)
/// 
fn eval(stack: &mut Vec<Pair<Rule>>, rule: Rule, tokens: &Vec<Token>) -> bool {
    trace!("eval.stack: {:?}", stack);

    let value = stack.pop().unwrap();           // simple value or comma separated value string....
    let op = stack.pop().unwrap();
    let type_term_arg = stack.pop().unwrap();   // n in type(n)
    let type_term = stack.pop().unwrap();       // date, time, timestamp, email, ... 

    // find n'th (type_term_arg) typeTerm among the token whose type == type_term..
    let valid_tokens:Vec<&Token> = tokens.into_iter()
        //.enumerate()
        .filter(|&token| token.is_match(type_term.as_str()))
        .collect();

    trace!("evail.token found {:?}", valid_tokens);
    let n = type_term_arg.as_str().parse::<usize>().unwrap();
    trace!("eval.type_term_arg {:?}", n);

    if !valid_tokens.is_empty() {
        match rule {
            Rule::simple_expr => return eval_op(op.as_rule(), value, valid_tokens[n-1]),
            Rule::contains_expr => return eval_op(op.as_rule(), value, valid_tokens[n-1]),
            _ => unreachable!("Unexpected rule matched!")
        }
    } 
    
    false
}

lazy_static! {
    static ref CLIMBER: PrecClimber<Rule> = {
        PrecClimber::new(vec![
            Operator::new(Rule::and_op, Assoc::Left) | Operator::new(Rule::or_op, Assoc::Left),
        ])    
    };
}

fn process_pair<'a>(pair: Pair<'a, Rule>, stack: &mut Vec<Pair<'a, Rule>>, tokens: &Vec<Token>) -> bool {

    let atom = |pair| process_pair(pair, stack, tokens);
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
        Rule::expr => { 
            return CLIMBER.climb(pair.into_inner(), atom, infix);
        },
        Rule::simple_expr => { 
            let _v: Vec<bool> = pair.into_inner().map(atom).collect();
            return eval(stack, Rule::simple_expr, tokens);
        },
        Rule::contains_expr => { 
            let _v: Vec<bool> = pair.into_inner().map(atom).collect();
            return eval(stack, Rule::simple_expr, tokens);
        },
        Rule::type_expr => { 
            let _v: Vec<bool> = pair.into_inner().map(atom).collect(); 
        },
        Rule::type_term => { 
            stack.push(pair); 
        },
        Rule::type_term_arg => { 
            stack.push(pair); 
        },
        Rule::op => { 
            stack.push(pair.into_inner().next().unwrap()); 
        },  
        Rule::in_op => { 
            stack.push(pair); 
        },  
        Rule::value => { 
            stack.push(pair); 
        },
        Rule::list_expr => {
            let _v: Vec<bool> = pair.into_inner().map(atom).collect(); 
        },
        Rule::list_member_expr => {
            stack.push(pair); 
        },
        _ => { 
            trace!("_: {}", pair.as_str());
        }
    }

    false
}

#[cfg(test)]
#[test]
fn test_parsing_should_pass() {
    use crate::{create_token};

    let tokens: Vec<Token> = vec![create_token("1970-07-31")];

    assert!(parse_expression("date(1) in [1970-07-31, now()]", &tokens));
    assert!(parse_expression("date(1) == 1970-07-31 && date(1) == 1970-07-31 || date(1) == 1970-07-31", &tokens));
    assert!(parse_expression("date(1) == 1970-07-31 && date(1) == 1970-07-31 || date(1) == 1970-07-30", &tokens));

    // these are negative tests....
    assert!( !parse_expression("date(1) == 1970-07-31 && date(1) == 1970-07-30 || date(1) == 1970-07-30", &tokens));
    assert!( !parse_expression("date(1) == 1900-01-01", &tokens));
}