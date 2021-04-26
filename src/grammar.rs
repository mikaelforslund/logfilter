

use pest::prec_climber::{Assoc, Operator, PrecClimber};
use pest::Parser;
use pest::iterators::Pairs;
use pest::iterators::Pair;
use lazy_static::lazy_static;
use crate::tokenizer::{Token, Token::DateToken};

#[derive(Parser)]
#[grammar = "pest_grammar.pest"]
struct SemFilterParser;

pub fn parse_expression(expr: &str, tokens: &Vec<Token>) -> bool {
    let mut grammar = SemFilterParser::parse(Rule::grammar, &expr)
        .unwrap_or_else(|e| panic!("{}", e));

//    println!("{:#?}", pairs);
    process_pair(grammar.next().unwrap(), &mut Vec::new(), tokens)
}

fn eval_op(op: Rule, value: &str, token: &Token) -> bool {
    match op {
        // TODO find the nth token as given by the typeTermArg
        eq => token == &token.new(value),
        neq => token != &token.new(value),
        lt => return token < &token.new(value),
        gt => return token > &token.new(value),
        lte => token <= &token.new(value),
        gte => token >= &token.new(value),
        
        // TODO
        //matchOp => {}
        //inOp => {}
        _ => return false  
    }
}

fn eval(stack: &mut Vec<Pair<Rule>>, rule: Rule, tokens: &Vec<Token>) -> bool {
    println!("stack: {:?}", stack);

    let value = stack.pop().unwrap();          // simple value or comma separated value string....
    let op = stack.pop().unwrap();
    let typeTermArg = stack.pop().unwrap();      // n in type(n)
    let typeTerm = stack.pop().unwrap();         // date, time, timestamp, email, ... 

    match rule {
        Rule::simpleExpr => {
            return eval_op(op.as_rule(), value.as_str(), &tokens[0])
        },
        //Rule::containsExpr => false
        _ => unreachable!("panic lah!!")
    }
    
    false
}

lazy_static! {
    static ref CLIMBER: PrecClimber<Rule> = {
        PrecClimber::new(vec![
            Operator::new(Rule::andOp, Assoc::Left) | Operator::new(Rule::orOp, Assoc::Left),
        ])    
    };
}

fn process_pair<'a>(pair: Pair<'a, Rule>, stack: &mut Vec<Pair<'a, Rule>>, tokens: &Vec<Token>) -> bool {

    let atom = |pair| process_pair(pair, stack, tokens);
    let infix = |lhs, op: Pair<Rule>, rhs| match op.as_rule() {
        Rule::andOp => {
            //let inner = pair.into_inner();
            println!("andOp: lhs: {}, rhs: {}", lhs, rhs);
            return lhs && rhs;
        },
        Rule::orOp => { 
            println!("orOp: lhs: {}, rhs: {}", lhs, rhs);
            return lhs || rhs;
        },
        _ => unreachable!(),
    };

    let inner_rule = pair.clone();
    println!("{:?} {:?}", inner_rule.as_rule(),  inner_rule.as_str());

    match pair.as_rule() {
        Rule::expr => {
            let pairs = pair.into_inner();
            return CLIMBER.climb(pairs, atom, infix);
        },
        Rule::simpleExpr => { 
            let v: Vec<bool> = pair.into_inner().map(atom).collect();
            return eval(stack, Rule::simpleExpr, tokens)
        },
        Rule::containsExpr => {
            return process_pair(pair.into_inner().next().unwrap(), stack, tokens);
        },    
        Rule::typeExpr => { 
            let v: Vec<bool> = pair.into_inner().map(atom).collect();
        },
        Rule::typeTerm => { 
            stack.push(pair);
        },
        Rule::typeTermArg => { 
            stack.push(pair);
        },
        Rule::op => {
            stack.push(pair);
        },  
        Rule::value => {
            stack.push(pair);
        },
        Rule::listExpr => {
            return process_pair(pair.into_inner().next().unwrap(), stack, tokens);
        },
        _ => {
            let inner = pair.into_inner();
            println!("_: {}", inner.as_str());
        }
    }

    false
}

#[cfg(test)]
#[test]
fn test_parsing() {
    let tokens: Vec<Token> = vec![DateToken("1970/07/31".to_string(), "yyyy/mm/dd".to_string())];

    // || date(1) > 1970/07/31 && date(*) in [1980/07/31, now()]", 

    //println!("{}", DateToken("1234".to_string(), "".to_string()) == DateToken("123".to_string(), "".to_string()));

    //  && date(1) == 1970/07/31 || date(1) == 1970/07/31
    assert!(parse_expression("date(1) == 1970/07/31 && date(1) == 1970/07/31 || date(1) == 1970/07/30", &tokens));

    assert!(!parse_expression("date(1) == 1900/01/01", &tokens));
}