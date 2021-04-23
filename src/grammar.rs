
use pest::Parser;
use pest::iterators::Pairs;
use pest::iterators::Pair;

use crate::parser::{Token, Token::DateToken};

#[derive(Parser)]
#[grammar = "pest_grammar.pest"]
struct SemFilterParser;

pub fn parse_expression(expr: &str, tokens: &Vec<Token>) -> bool {
    let pairs = SemFilterParser::parse(Rule::grammar, &expr)
        .unwrap_or_else(|e| panic!("{}", e));

//    println!("{:#?}", pairs);
    process_pairs(pairs, &mut Vec::new(), tokens)
}

fn eval_op(token: &Token, rule: &Pair<Rule>) -> bool {
    match token {
        Token::DateToken(v, f) => v == rule.as_str(),

        // TODO implement these...
        Token::StringToken(v) => { return true },
        Token::IntegerToken(v) => { return true },
        Token::NumberToken(v) => { return true },
        Token::EmailToken(v) => { return true },
        Token::IPv4Token(v) => { return true },
        Token::IPv6Token(v) => { return true },
        _ => unreachable!()
    }
}

fn eval(stack: &mut Vec<Pair<Rule>>, rule: Rule, tokens: &Vec<Token>) -> bool {
    match rule {
        Rule::simpleExpr => {
            // stack should have: 
            //   pop => value
            //   pop => op
            //   pop => typeTermArg
            //   pop => typeTerm

            println!("stack: {:?}", stack);

            let value = stack.pop();
            let op = stack.pop().unwrap();
            let typeTermArg = stack.pop();
            let typeTerm = stack.pop();

            match op {
                // TODO find the nth token as given by the typeTermArg
                eq => { 
                    //println!("value: {:?}", value.unwrap().as_str());
                    //println!("token: {:?}", tokens[0]);
                 
                    return eval_op(&tokens[0], &value.unwrap());
                },
                neq => {},
                lt => {},
                gt => {},
                lte => {},
                gte => {},
                matchOp => {},
                inOp => {}
                _ => { println!("{:?}", op) } //unreachable!("ignore for now")
            }
        },
        Rule::containsExpr => {}
        _ => unreachable!("panic lah!!")
    }
    
    false
}

fn process_pairs<'a>(pairs: Pairs<'a, Rule>, stack: &mut Vec<Pair<'a, Rule>>, tokens: &Vec<Token>) -> bool {

    for pair in pairs {
        match pair.as_rule() {
            Rule::simpleExpr => { 
                let inner = pair.into_inner();
                println!("simpleExpr: {}", inner.as_str());
                process_pairs(inner, stack, tokens);
                return eval(stack, Rule::simpleExpr, tokens)
            },
            Rule::typeExpr => { 
                let inner = pair.into_inner();
                println!("typeExpr: {}", inner.as_str());               
                process_pairs(inner, stack, tokens);
            },
            Rule::typeTerm => { 
                println!("typeTerm: {}", pair.as_str());
                stack.push(pair);
            },
            Rule::typeTermArg => { 
                println!("typeTermArg: {}", pair.as_str());
                stack.push(pair);
            },
            Rule::op => {
                println!("op: {}", pair.as_str());
                stack.push(pair);
            },  
            Rule::value => {
                 println!("value: {}", pair.as_str());
                 stack.push(pair);
                },
            Rule::andExpr => {
                let inner = pair.into_inner();
                println!("andExpr: {}", inner.as_str());
                process_pairs(inner, stack, tokens);
            },
            Rule::orExpr => { 
                let inner = pair.into_inner();
                println!("orExpr: {}", inner.as_str());
                process_pairs(inner, stack, tokens);
            },
            Rule::listExpr => {
                let inner = pair.into_inner();
                println!("listExpr: {}", inner.as_str());
                process_pairs(inner, stack, tokens);
            }
            Rule::containsExpr => {
                let inner = pair.into_inner();
                println!("containsExpr: {}", inner.as_str());
                process_pairs(inner, stack, tokens);
            },    
            _ => {
                let inner = pair.into_inner();
                println!("_: {}", inner.as_str());
            }
        } 
    }

    false // TODO return the real evaluation...
}

#[cfg(test)]
#[test]
fn test_parsing() {
    let tokens: Vec<Token> = vec![DateToken("1970/07/31".to_string(), "yyyy/mm/dd".to_string())];

    // || date(1) > 1970/07/31 && date(*) in [1980/07/31, now()]", 
    assert!(parse_expression("date(1) == 1970/07/31", &tokens));

    assert!(!parse_expression("date(1) == 1900/01/01", &tokens));
}