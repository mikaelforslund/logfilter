
use clap::{App, ArgMatches};
use std::default::Default;
use regex::Regex;

#[derive(Debug, Default)]
pub struct DataDef {
    type_name: String,
    format: String
}

#[derive(Debug, Default)]
pub struct CommandArgs {
    pub expr: String, 
    pub data_def: Vec<DataDef>,  
    pub token_regex: Option<Regex>
}

pub fn parse_cli() -> CommandArgs { 
    let yaml = load_yaml!("cli.yaml");
    let matches = App::from_yaml(yaml).get_matches();
    
    CommandArgs { 
        expr: matches.value_of("expr").unwrap().to_string(), 
        data_def: get_data_def(&matches), 
        token_regex: get_token_sep(&matches)
    }
} 

fn get_data_def(matches: &ArgMatches) -> Vec<DataDef> { 
    if let Some(data_defs) = matches.value_of("data-def") {
        // e.g. date|yyyy/mm/dd, string|regexp, ...
        data_defs.split(",").into_iter()
            .map(|def| { 
                let type_def:Vec<&str> = def.split("|").collect();
                DataDef{ type_name: String::from(type_def[0]), format: String::from(type_def[1]) }
            }).collect::<Vec<DataDef>>()
    } else { 
        Default::default() 
    }
}

fn get_token_sep(matches: &ArgMatches) -> Option<Regex>{
    if let Some(token_separators) = matches.value_of("token-sep") {
        // e.g. "," or "<>" or " "
        Some(Regex::new(token_separators).unwrap())
    } else { 
        None 
    }
}

#[cfg(test)]
mod tests {   
    mod tests {
        use clap::{App};
        use crate::cli::*;

        #[test]
        fn test_simple_args() {                     
            let arg_vec = vec!["semfilter", "date(0) == 1900-01-01", "--token-sep=\",|\"", "--data-def=date|yyyy/MM/dd"];
            let _target_vec= vec![DataDef{type_name:String::from("date"), format:String::from("yyyy/MM/dd")}];

            let yaml = load_yaml!("cli.yaml");
            let matches = App::from_yaml(yaml).get_matches_from(arg_vec);
            
            let data_defs = get_data_def(&matches);
            let token_sep = get_token_sep(&matches);

            assert!(matches!(data_defs, _target_vec));
            assert!(token_sep.is_some());            

            //println!("data_defs: {:?}", data_defs);
            //println!("token_sep: {:?}", token_sep);
        }
    }

    #[test]
    fn misc_test() {
    }
}