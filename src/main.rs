use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Read;
use std::process::Command;

use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about,
    long_about = "json_env reads the .env.json file in the current directory and runs a program with these environment variables."
)]
struct Args {
    /// Should env variables be extended
    #[arg(short, long, default_value_t = false)]
    expand: bool,
    #[arg(short, long, default_value = ".env.json")]
    config_file: String,
    /// The executable which should be started, with it's command line arguments.
    executable: Vec<String>,
}

// `json_env` is [dotenv](https://github.com/motdotla/dotenv), but with JSON.
// See [readme](Readme.md) for more information.
fn main() {
    let args: Args = Args::parse();
    if args.executable.is_empty() {
        eprintln!("You need to provide the name of an executable!");
        return;
    }

    match File::open(args.config_file) {
        Ok(mut file) => {
            let mut contents = String::new();
            if file.read_to_string(&mut contents).is_ok() {
                match parse_json(&contents, args.expand) {
                    Ok(res) => {
                        execute(&res, &args.executable[0], &(args.executable[1..]).to_vec());
                    }
                    Err(_) => {
                        eprintln!("Could not parse content of {{args.config_file}}!");
                    }
                }
            }
        }
        Err(_) => {
            eprintln!("Could not open the .env.json file. Make sure it exists in the current directory and can be read.");
        }
    }
}

fn execute(vars: &HashMap<String, String>, command: &str, args: &Vec<String>) {
    match Command::new(command).envs(vars).args(args).spawn() {
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

fn parse_json(in_str: &str, should_expand: bool) -> Result<HashMap<String, String>> {
    let completely_parsed: HashMap<String, serde_json::Value> = serde_json::from_str(in_str)?;
    let mut only_strings = HashMap::new();
    for (key, val) in completely_parsed.iter() {
        let mut val_str = "".to_string();
        if val.is_array() {
            val_str = val.to_string();
        }
        if val.is_boolean() {
            val_str = val.to_string();
        }
        if val.is_object() {
            val_str = val.to_string();
        }
        if val.is_null() {
            val_str = val.to_string();
        }
        if val.is_number() {
            val_str = val.to_string();
        }
        if val.is_string() {
            val_str = val.as_str().unwrap().to_string(); // the as_str is needed, because we get quotes otherwise
        }
        if should_expand {
            let mut expanded_val = val_str.clone();
            for (env_key, env_value) in env::vars() {
                let env_key_dollar = format!("${env_key}");
                if val_str.contains(&env_key_dollar) {
                    expanded_val = val_str.replace(&env_key_dollar, &env_value);
                }
            }
            only_strings.insert(key.to_string(), expanded_val);
        } else {
            only_strings.insert(key.to_string(), val_str);
        }
    }
    Ok(only_strings)
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn parse_simple() {
        let simple_json = include_str!("../examples/simple/.env.json");
        let result = parse_json(&simple_json, false);
        match result {
            Ok(res) => {
                let node_env = res.get("NODE_ENV");
                assert!(node_env.is_some());
                assert_eq!(node_env.unwrap().to_string(), "DEV".to_string());
            }
            Err(e) => panic!("{e}"),
        }
    }

    #[test]
    fn expand() {
        let simple_json = include_str!("../examples/extend.json");
        env::set_var("FOO", "Bar");
        let result = parse_json(&simple_json, true);
        match result {
            Ok(res) => {
                let node_env = res.get("TEST");
                assert!(node_env.is_some());
                assert_eq!(node_env.unwrap().to_string(), "Bar".to_string());
            }
            Err(e) => panic!("{e}"),
        }
    }

    #[test]
    fn expand_nested() {
        let simple_json = include_str!("../examples/extend2.json");
        env::set_var("FOO", "Bar");
        let result = parse_json(&simple_json, true);
        match result {
            Ok(res) => {
                let node_env = res.get("TEST");
                assert!(node_env.is_some());
                assert!(node_env.unwrap().contains("Bar"));
            }
            Err(e) => panic!("{e}"),
        }
    }
}
