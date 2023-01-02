use anyhow::{Error, Result};
use clap::error::ErrorKind;
use clap::CommandFactory;
use clap::Parser;
use dialoguer::Confirm;
use dirs::home_dir;
use jsonpath_rust::JsonPathFinder;
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::Command;
use std::string::ToString;
use std::{env, fs, process};

struct Shell {
    shell_type: ShellType,
    config_path: &'static str,
    script: &'static str,
}

#[derive(Debug)]
enum ShellType {
    Bash,
    Zsh,
    Fish,
}

impl Display for ShellType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ShellType::Bash => write!(f, "bash"),
            ShellType::Zsh => write!(f, "zsh"),
            ShellType::Fish => write!(f, "fish"),
        }
    }
}

impl Clone for ShellType {
    fn clone(&self) -> Self {
        match self {
            ShellType::Bash => ShellType::Bash,
            ShellType::Zsh => ShellType::Zsh,
            ShellType::Fish => ShellType::Fish,
        }
    }
}

impl Copy for ShellType {}

impl Clone for Shell {
    fn clone(&self) -> Self {
        Shell {
            shell_type: self.shell_type,
            config_path: self.config_path,
            script: self.script,
        }
    }
}

impl Display for Shell {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.shell_type)
    }
}

static BASH: Shell = Shell {
    shell_type: ShellType::Bash,
    config_path: ".bash_profile",
    script: include_str!("run_on_cd.bash"),
};

static FISH: Shell = Shell {
    shell_type: ShellType::Fish,
    config_path: ".config/fish/config.fish",
    script: include_str!("run_on_cd.fish"),
};

static ZSH: Shell = Shell {
    shell_type: ShellType::Zsh,
    config_path: ".zshrc",
    script: include_str!("run_on_cd.zsh"),
};

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
    /// Do not run an application but print export commands. (can be sourced)
    #[arg(long, default_value_t = false)]
    export: bool,
    /// The JSON files from which the environment variables are taken from
    #[arg(short, long)]
    config_files: Vec<String>,
    /// A JSON paths into the config files, in order. For examples and spec, see https://docs.rs/jsonpath-rust/latest/jsonpath_rust/
    #[arg(short, long, default_value = "$")]
    paths: Vec<String>,
    /// The executable which should be started, with it's command line arguments.
    executable: Vec<String>,
    /// add a script to your shell configuration that automatically exports variables defined in .env.json when changing into a directory that contains such a file.
    #[arg(long, default_value_t = false)]
    install: bool,
    /// Silent mode, do not report errors (useful for scripts). Implies 'yes' to all questions.
    #[arg(short, long, default_value_t = false)]
    silent: bool,
    /// Print the path of the .env.json file that is used.
    #[arg(long, default_value_t = false)]
    print_config_path: bool,
}

// `json_env` is [dotenv](https://github.com/motdotla/dotenv), but with JSON.
// See [readme](Readme.md) for more information.
fn main() {
    let mut args: Args = Args::parse();
    let mut cmd = Args::command();


    if args.install {
        install_shell_script(args.silent);
        return;
    }

    let mut env_vars: HashMap<String, String> = HashMap::new();
    if args.print_config_path {
        if let Some(config_path) = find_env_file() {
            println!("{}", config_path.to_str().unwrap());
        } else {
            println!("No .env.json file found");
        }
        return;
    }

    if args.executable.is_empty() && !args.export {
        if args.silent {
            process::exit(1);
        }
        cmd.error(
            ErrorKind::TooFewValues,
            "You need to provide the name of an executable",
        )
            .exit();
    }

    if args.config_files.is_empty() {
        let env_json = find_env_file();
        if let Some(path) = env_json {
            args.config_files.push(path.to_str().unwrap().to_string());
        } else if args.silent {
            process::exit(1);
        } else {
            cmd.error(
                ErrorKind::TooFewValues,
                "You need to provide the name of a config file or be in a directory with a .env.json file in it or one of it's parents",
            )
                .exit();
        }
    }

    // Read the config files, and parse them as JSON
    for (i, file_name) in args.config_files.iter().enumerate() {
        let Ok(mut file) = File::open(file_name) else {
            if args.silent {
                return;
            }
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
                        if args.silent {
                            return;
                        }
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
                Err(e) => {
                    if args.silent {
                        return;
                    }
                    cmd.error(
                        ErrorKind::InvalidValue,
                        format!("error while parsing json or jsonpath:  {}", e),
                    )
                        .exit()
                }
            }
        } else {
            if args.silent {
                return;
            }
            cmd.error(
                ErrorKind::InvalidValue,
                format!("Could not read JSON in '{}'", file_name),
            )
                .exit();
        }
    }

    if args.export {
        for (k, v) in &env_vars {
            println!("export {}=\"{}\"", k, v);
        }
        return;
    }

    execute(
        &env_vars,
        &args.executable[0],
        &(args.executable[1..]).to_vec(),
    )
}

fn install_shell_script(silent: bool) {
    let Ok(shell) = get_shell() else {
        if !silent {
            println!("Could not determine shell");
        }
        process::exit(1);
    };

    // Get the appropriate script to append to the configuration file
    // for the user's current shell
    if !silent
        && !Confirm::new()
        .with_prompt(format!(
            "Your shell has been detected as: '{}', is that correct?",
            shell
        ))
        .interact()
        .unwrap_or(false)
    {
        println!("Please set your shell to one of the supported shells and try again.");
        return;
    }

    // indent the script string using 4 spaces
    let script = shell
        .script
        .lines()
        .map(|line| format!("    {}", line))
        .collect::<Vec<String>>()
        .join("\n");

    let config_path = match home_dir() {
        Some(mut path) => {
            path.push(shell.config_path);
            path
        }
        None => {
            if !silent {
                println!("Could not determine home directory");
            }
            process::exit(1);
        }
    };
    if !silent {
        println!(
            "I am going to append the following lines to your shell configuration file at '{}':\n {}\n",
            config_path.to_str().unwrap(),
            script
        );
    }

    if !silent
        && !Confirm::new()
        .with_prompt("Do you want me to do that? ")
        .interact()
        .unwrap_or(false)
    {
        println!("Please set your shell to one of the supported shells and try again.");
        return;
    }

    // Check if the config file exists and create it if it does not
    if !config_path.exists() {
        let Ok(mut file) = File::create(&config_path) else {
            if !silent {
                println!("Could not create file '{}'", config_path.to_str().unwrap());
            }
            process::exit(1);
        };
        file.write_all(b"").unwrap();
    }

    // Open the configuration file in append-only mode
    let mut file = match fs::OpenOptions::new().append(true).open(&config_path) {
        Ok(file) => file,
        Err(error) => {
            if !silent {
                println!(
                    "Could not open file '{}': {}",
                    config_path.to_str().unwrap(),
                    error
                );
            }
            process::exit(1);
        }
    };

    if let Err(error) = file.write_all(shell.script.as_bytes()) {
        if !silent {
            println!(
                "Could not write to file '{}': {}",
                config_path.to_str().unwrap(),
                error
            );
        }
        process::exit(1);
    }
}

/// Get the the user's current shell
fn get_shell() -> Result<Shell, String> {
    // Get the name of the user's current shell
    let shell_path = match env::var("SHELL") {
        Ok(shell) => PathBuf::from(shell),
        Err(error) => {
            return Err(format!("Error getting shell name: {}", error));
        }
    };
    let Some(file_name_os_str) = shell_path.file_name() else {
        return Err("Error getting shell name".to_string());
    };
    let Some(file_name) = file_name_os_str.to_str() else {
        return Err("Error getting shell name".to_string());
    };
    match file_name {
        "bash" => Ok(BASH.clone()),
        "zsh" => Ok(ZSH.clone()),
        "fish" => Ok(FISH.clone()),
        _ => Err("Unknown shell".to_string()),
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

/// Execute the given command with the given environment variables.
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

/// Recursively find the file '.env.json' in the current directory and all parent directories.
fn find_env_file() -> Option<PathBuf> {
    let mut current_dir = env::current_dir().unwrap();
    loop {
        let mut env_file = current_dir.clone();
        env_file.push(".env.json");
        if env_file.exists() {
            return Some(env_file);
        }
        if !current_dir.pop() {
            return None;
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
        let val = parse_and_extract(simple_json, "$");
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
        let val = parse_and_extract(extendable_json, "$");
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
        let val = parse_and_extract(extendable_json, "$");
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
        let val = parse_and_extract(nested_json, "$.nested");
        assert!(val.is_ok());
        let mut env_vars: HashMap<String, String> = HashMap::new();
        add_values_to_map(&val.unwrap(), true, &mut env_vars);
        let hello = env_vars.get("hello");
        assert!(hello.is_some());
        assert_eq!(hello.unwrap(), "world");
    }
}
