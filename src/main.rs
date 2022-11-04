use anyhow::{Error, Result};
use clap::error::ErrorKind;
use clap::CommandFactory;
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
    config_files: Vec<String>,
    /// A JSON paths into the config files, in order. For examples and spec, see https://docs.rs/jsonpath-rust/latest/jsonpath_rust/
    #[arg(short, long, default_value = "$")]
    paths: Vec<String>,
    /// The executable which should be started, with it's command line arguments.
    executable: Vec<String>,
}

// `json_env` is [dotenv](https://github.com/motdotla/dotenv), but with JSON.
// See [readme](Readme.md) for more information.
fn main() {
    let args: Args = Args::parse();
    let mut cmd = Args::command();
    if args.executable.is_empty() {
        cmd.error(
            ErrorKind::TooFewValues,
            "You need to provide the name of an executable",
        )
        .exit();
    }

    let mut env_vars: HashMap<String, String> = HashMap::new();

    for (i, file_name) in args.config_files.iter().enumerate() {
        let Ok(mut file) = File::open(file_name) else {
            cmd.error(
                ErrorKind::InvalidValue,
                format!("Could not open '{}'", file_name),
            ).exit();
        };
        let mut contents = String::new();
        let json_path = match args.paths.get(i) {
            Some(p) => p,
            None => "$",
        };
        if file.read_to_string(&mut contents).is_ok() {
            match parse_and_extract(&contents, json_path) {
                Ok(val) => {
                    if val.is_empty() {
                        cmd.error(
                            ErrorKind::InvalidValue,
                            format!(
                                "There is nothing in file '{}' at path '{}'",
                                file_name, json_path
                            ),
                        )
                        .exit();
                    }
                    add_values_to_map(&val, args.expand, &mut env_vars);
                }
                Err(e) => cmd
                    .error(
                        ErrorKind::InvalidValue,
                        format!("error while parsing json or jsonpath:  {}", e),
                    )
                    .exit(),
            }
        } else {
            cmd.error(
                ErrorKind::InvalidValue,
                format!("Could not read JSON in '{}'", file_name),
            )
            .exit();
        }
    }
    execute(
        &env_vars,
        &args.executable[0],
        &(args.executable[1..]).to_vec(),
    )
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

fn add_values_to_map(
    values: &Vec<Value>,
    should_expand: bool,
    str_map: &mut HashMap<String, String>,
) {
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
                    str_map.insert(key.to_string(), expanded_val);
                } else {
                    str_map.insert(key.to_string(), val_str);
                }
            }
        }
    }
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
        let mut env_vars: HashMap<String, String> = HashMap::new();
        add_values_to_map(&val.unwrap(), false, &mut env_vars);
        let node_env = env_vars.get("NODE_ENV");
        assert!(node_env.is_some());
        assert_eq!(node_env.unwrap().to_string(), "DEV".to_string());
    }

    #[test]
    fn expand() {
        let extendable_json = include_str!("../examples/extend.json");
        env::set_var("FOO", "Bar");
        let val = parse_and_extract(&extendable_json, "$");
        assert!(val.is_ok());
        let mut env_vars: HashMap<String, String> = HashMap::new();
        add_values_to_map(&val.unwrap(), true, &mut env_vars);
        let node_env = env_vars.get("TEST");
        assert!(node_env.is_some());
        assert_eq!(node_env.unwrap().to_string(), "Bar".to_string());
    }

    #[test]
    fn expand_nested() {
        let extendable_json = include_str!("../examples/extend2.json");
        env::set_var("FOO", "Bar");
        let val = parse_and_extract(&extendable_json, "$");
        assert!(val.is_ok());
        let mut env_vars: HashMap<String, String> = HashMap::new();
        add_values_to_map(&val.unwrap(), true, &mut env_vars);
        let node_env = env_vars.get("TEST");
        assert!(node_env.is_some());
        assert!(node_env.unwrap().contains("Bar"));
    }

    #[test]
    fn use_json_path() {
        let nested_json = include_str!("../examples/nested/.env.json");
        let val = parse_and_extract(&nested_json, "$.nested");
        assert!(val.is_ok());
        let mut env_vars: HashMap<String, String> = HashMap::new();
        add_values_to_map(&val.unwrap(), true, &mut env_vars);
        let hello = env_vars.get("hello");
        assert!(hello.is_some());
        assert_eq!(hello.unwrap(), "world");
    }
}
