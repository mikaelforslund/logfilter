
use clap::App;
use std::default::Default;

#[derive(Debug, Default)]
pub struct DataDef {
    type_name: String,
    format: String
}

#[derive(Debug, Default)]
pub struct CommandArgs {
    pub expr: String, 
    pub data_def: Vec<DataDef>,  
    pub token_sep: Vec<String>
}

pub fn parse_cli() -> CommandArgs { 
    let yaml = load_yaml!("cli.yaml");
    let matches = App::from_yaml(yaml).get_matches();
    
    let data_defs: Vec<DataDef> = if let Some(data_defs) = matches.value_of("data_def") {
        // e.g. date|yyyy/mm/dd, string|regexp, ...
        data_defs.split(",").into_iter()
            .map(|def| { 
                let type_def:Vec<&str> = def.split("|").collect();
                DataDef{ type_name: String::from(type_def[0]), format: String::from(type_def[1]) }
            }).collect::<Vec<DataDef>>()
    } else { 
        Default::default() 
    };

    let token_sep: Vec<String> = if let Some(token_separators) = matches.values_of("token_sep") {
        // e.g. "," or "<>" or " "
        token_separators.map(|str| String::from(str)).collect::<Vec<String>>()
    } else { 
        Default::default() 
    };
    
    // TODO rest of the expressions
    CommandArgs { 
        expr: matches.value_of("expr").unwrap().to_string(), 
        data_def: data_defs, 
        token_sep: token_sep 
    }
} 