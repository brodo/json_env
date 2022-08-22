use std::fs::File;
use std::io::Read;
use anyhow::Result;
use std::collections::HashMap;
use std::env;
use std::process::{Command};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("json_env reads the .env.json file in the current directory\nand runs a program with these environment variables.\n");
        println!("Usage:");
        println!("json_env <executable> <options for executable>\n");
        println!("json_env itself has no config options");
    }

    match File::open(".env.json") {
        Ok(mut file) => {
            let mut contents = String::new();
            if let Ok(_) = file.read_to_string(&mut contents) {
                let cmd_args: Vec<String> = args[2..].to_vec();
                match parse_json(&contents) {
                    Ok(res) => {
                        execute(&res, &args[1], &cmd_args);
                    }
                    Err(_) => {
                        eprintln!("Could not parse content of .env.json!");
                    }
                }
            }
        }
        Err(_) => {
            eprintln!("Could not open the .env.json file. Make sure it exists in the\ncurrent directory and can be read.");
        }
    }
}

fn execute(vars: &HashMap<String, String>, command: &str, args: &Vec<String>) {
    match Command::new(command)
        .envs(vars)
        .args(args)
        .spawn() {
        Err(e) => {
            eprintln!("Could not start executable '{command}': {e}");
        }
        Ok(mut child) => {
            if let Err(e) = child.wait() {
                eprintln!("Error when running executable '{command}: {e}");
            }
        }
    }
}


fn parse_json(in_str: &str) -> Result<HashMap<String, String>> {
    let completely_parsed: HashMap<String, serde_json::Value> = serde_json::from_str(&in_str)?;
    let mut only_strings = HashMap::new();
    for (str, val) in completely_parsed.iter() {
        let mut val_str = "".to_string();
        if val.is_array() {
            val_str = val.to_string();
        }
        if val.is_boolean() {
            val_str = val.to_string();
        }
        if val.is_object() {
            val_str = format!("{}", val.to_string());
        }
        if val.is_null() {
            val_str = val.to_string();
        }
        if val.is_number() {
            val_str = val.to_string();
        }
        if val.is_string() {
            val_str = format!("'{}'", val.as_str().unwrap());
        }
        only_strings.insert(str.to_string(), val_str);
    }
    Ok(only_strings)
}
