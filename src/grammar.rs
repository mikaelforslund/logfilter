

use pest::prec_climber::{Assoc, Operator, PrecClimber};
use pest::Parser;
use pest::iterators::Pair;
use lazy_static::lazy_static;
use crate::tokenizer::{Token, Token::DateToken, create_token};
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use std::str::FromStr;

#[derive(Parser)]
#[grammar = "pest_grammar.pest"]
struct SemFilterParser;

pub fn parse_expression(expr: &str, tokens: &Vec<Token>) -> bool {
    let mut grammar = SemFilterParser::parse(Rule::grammar, &expr)
        .unwrap_or_else(|e| panic!("{}", e));

//    println!("{:#?}", pairs);
    process_pair(grammar.next().unwrap(), &mut Vec::new(), tokens)
}

fn eval_op(op: Rule, value: Pair<Rule>, token: &Token) -> bool {
    match op {
        // TODO find the nth token as given by the typeTermArg
        Rule::eq => token == &token.new(value.as_str()),
        Rule::neq => token != &token.new(value.as_str()),
        Rule::lt => return token < &token.new(value.as_str()),
        Rule::gt => return token > &token.new(value.as_str()),
        Rule::lte => token <= &token.new(value.as_str()),
        Rule::gte => token >= &token.new(value.as_str()),
        
        // TODO
        //match_op => {},
        Rule::in_op =>  {             
            println!("value: {:?}", value);
            let tokens:Vec<Token> = value.into_inner().map(|rule| token.new(rule.as_str())).collect();
            println!("tokens: {:?}", tokens);
            return tokens.contains(&token);
        },
        _ => {
            return false  
        }
    }
}

fn eval(stack: &mut Vec<Pair<Rule>>, rule: Rule, tokens: &Vec<Token>) -> bool {
    println!("stack: {:?}", stack);

    let value = stack.pop().unwrap();          // simple value or comma separated value string....
    let op = stack.pop().unwrap();

    // TODO use this data to find the correct token...
    let type_term_arg = stack.pop().unwrap();      // n in type(n)
    let type_term = stack.pop().unwrap();         // date, time, timestamp, email, ... 

    match rule {
        Rule::simple_expr => return eval_op(op.as_rule(), value, &tokens[0]),

        // TODO turn the value into a list so that we can checl membership using 'in'
        Rule::contains_expr => return eval_op(op.as_rule(), value, &tokens[0]),
        _ => unreachable!("panic lah!!")
    }
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
            //let inner = pair.into_inner();
            println!("andOp: lhs: {}, rhs: {}", lhs, rhs);
            return lhs && rhs;
        },
        Rule::or_op => { 
            println!("orOp: lhs: {}, rhs: {}", lhs, rhs);
            return lhs || rhs;
        },
        _ => unreachable!(),
    };

    let inner_rule = pair.clone();
    println!("{:?} {:?}", inner_rule.as_rule(),  inner_rule.as_str());

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
            println!("_: {}", pair.as_str());
        }
    }

    false
}

#[cfg(test)]
#[test]
fn test_parsing_should_pass() {
    let tokens: Vec<Token> = vec![create_token("1970-07-31")];

    assert!(parse_expression("date(1) in [1970-07-31, now()]", &tokens));

    assert!(parse_expression("date(1) == 1970-07-31 && date(1) == 1970-07-31 || date(1) == 1970-07-31", &tokens));

    assert!(parse_expression("date(1) == 1970-07-31 && date(1) == 1970-07-31 || date(1) == 1970-07-30", &tokens));

    assert!(!parse_expression("date(1) == 1970-07-31 && date(1) == 1970-07-30 || date(1) == 1970-07-30", &tokens));
 
    assert!(!parse_expression("date(1) == 1900-01-01", &tokens));
}