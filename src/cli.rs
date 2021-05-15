
use clap::App;
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
    
    let data_defs: Vec<DataDef> = if let Some(data_defs) = matches.value_of("data-def") {
        // e.g. date|yyyy/mm/dd, string|regexp, ...
        data_defs.split(",").into_iter()
            .map(|def| { 
                let type_def:Vec<&str> = def.split("|").collect();
                DataDef{ type_name: String::from(type_def[0]), format: String::from(type_def[1]) }
            }).collect::<Vec<DataDef>>()
    } else { 
        Default::default() 
    };

    let token_regex: Option<Regex> = if let Some(token_separators) = matches.value_of("token-sep") {
        // e.g. "," or "<>" or " "
        Some(Regex::new(token_separators).unwrap()) //token_separators.map(|str| String::from(str)).collect::<Vec<String>>()
    } else { 
        None 
    };
    
    CommandArgs { 
        expr: matches.value_of("expr").unwrap().to_string(), 
        data_def: data_defs, 
        token_regex: token_regex 
    }
} 

#[cfg(test)]
mod tests {   
    mod tests {
        //use clap::App;

        // TODO
    //     #[test]
    //     fn test_simple_args() { 
    //         let yaml = load_yaml!("cli.yaml");

    //         let matches = App::from_yaml(yaml).get_matches();

    //         println!("{:?}", matches);
    //     }
    }
}