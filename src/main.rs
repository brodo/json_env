use anyhow::{Error, Result};
use clap::Parser;
use jsonpath_rust::JsonPathFinder;
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Read;
use std::process::Command;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Reads a JSON file and runs a program with these environment variables."
)]
struct Args {
    /// Expand env variables
    #[arg(short, long, default_value_t = false)]
    expand: bool,
    /// The JSON files from which the environment variables are taken from
    #[arg(short, long, default_value = ".env.json")]
    config_file: String,
    /// A JSON path into the config. For examples and spec, see https://docs.rs/jsonpath-rust/latest/jsonpath_rust/
    #[arg(short, long, default_value = "$")]
    path: String,
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
                match parse_and_extract(&contents, &args.path) {
                    Ok(val) => {
                        let hash_map = value_to_hash_map(&val, args.expand);
                        execute(
                            &hash_map,
                            &args.executable[0],
                            &(args.executable[1..]).to_vec(),
                        )
                    }

                    Err(e) => eprintln!("error while parsing json or jsonpath: {}", e),
                }
            }
        }
        Err(_) => {
            eprintln!("Could not open the .env.json file. Make sure it exists in the current directory and can be read.");
        }
    }
}

fn parse_and_extract(json_str: &str, path: &str) -> Result<Vec<Value>> {
    let finder = JsonPathFinder::from_str(json_str, path).map_err(Error::msg)?;
    finder
        .find()
        .as_array()
        .cloned()
        .ok_or_else(|| Error::msg("Json path does not point to valid object."))
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

fn value_to_hash_map(values: &Vec<Value>, should_expand: bool) -> HashMap<String, String> {
    let mut only_strings = HashMap::new();
    for value in values {
        if value.is_object() {
            let in_val = value.as_object().unwrap();
            for (key, val) in in_val {
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
        }
    }
    only_strings
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn parse_simple() {
        let simple_json = include_str!("../examples/simple/.env.json");
        let val = parse_and_extract(&simple_json, "$");
        assert!(val.is_ok());
        let result = value_to_hash_map(&val.unwrap(), false);
        let node_env = result.get("NODE_ENV");
        assert!(node_env.is_some());
        assert_eq!(node_env.unwrap().to_string(), "DEV".to_string());
    }

    #[test]
    fn expand() {
        let extendable_json = include_str!("../examples/extend.json");
        env::set_var("FOO", "Bar");
        let val = parse_and_extract(&extendable_json, "$");
        assert!(val.is_ok());
        let result = value_to_hash_map(&val.unwrap(), true);
        let node_env = result.get("TEST");
        assert!(node_env.is_some());
        assert_eq!(node_env.unwrap().to_string(), "Bar".to_string());
    }

    #[test]
    fn expand_nested() {
        let extendable_json = include_str!("../examples/extend2.json");
        let val = parse_and_extract(&extendable_json, "$");
        assert!(val.is_ok());
        let result = value_to_hash_map(&val.unwrap(), true);
        let node_env = result.get("TEST");
        assert!(node_env.is_some());
        assert!(node_env.unwrap().contains("Bar"));
    }

    #[test]
    fn use_json_path() {
        let nested_json = include_str!("../examples/nested/.env.json");
        let val = parse_and_extract(&nested_json, "$.nested");
        assert!(val.is_ok());
        let result = value_to_hash_map(&val.unwrap(), true);
        let hello = result.get("hello");
        assert!(hello.is_some());
        assert_eq!(hello.unwrap(), "world");
    }
}
