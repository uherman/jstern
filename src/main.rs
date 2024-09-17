use colored::*;
use colored_json::prelude::*;
use ctrlc;
use serde_json::{Map, Value};
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::thread;
use term_size;

struct Args {
    pod_query: String,
    namespace: Option<String>,
    selector: Option<String>,
    keys: Vec<String>,
    filters: Vec<(String, String)>,
    separator: bool,
    padding: bool,
}

impl Args {
    fn new(args: Vec<String>) -> Self {
        if args.len() < 2
            || args.contains(&"-h".to_string())
            || args.contains(&"--help".to_string())
        {
            Args::print_help();
            std::process::exit(0);
        }

        let mut namespace = None;
        let mut selector = None;
        let mut keys = Vec::new();
        let mut filters = Vec::new();
        let mut separator = false;
        let mut padding = false;
        let mut i = 2;

        while i < args.len() {
            match args[i].as_str() {
                "-n" | "--namespace" => {
                    namespace = Some(Self::get_next_arg(&args, i));
                    i += 1;
                }
                "-s" | "--selector" => {
                    selector = Some(Self::get_next_arg(&args, i));
                    i += 1;
                }
                "-k" | "--keys" => {
                    i += 1;
                    if i >= args.len() || args[i].starts_with('-') {
                        panic!("{}", "No keys provided for -k argument".red());
                    }
                    while i < args.len() && !args[i].starts_with('-') {
                        keys.push(args[i].clone());
                        i += 1;
                    }
                    i -= 1;
                }
                "-f" | "--filter" => {
                    if i + 2 < args.len() {
                        filters.push((args[i + 1].clone(), args[i + 2].clone()));
                        i += 2;
                    } else {
                        panic!(
                            "{}",
                            "No filter provided for -f argument. Usage: -f key value".red()
                        );
                    }
                }
                "--separator" => separator = true,
                "--padding" => padding = true,
                _ => {}
            }
            i += 1;
        }

        Args {
            pod_query: args[1].clone(),
            namespace,
            selector,
            keys,
            filters,
            separator,
            padding,
        }
    }

    fn get_next_arg(args: &[String], index: usize) -> String {
        if index + 1 < args.len() {
            args[index + 1].clone()
        } else {
            panic!(
                "{}",
                format!("Missing value for argument {}", args[index]).red()
            );
        }
    }

    fn print_help() {
        println!("{}", "Usage: jstern <pod_query> [options]".green());
        println!("{}", "Options:".green());
        println!("  -n, --namespace <namespace>     Specify the Kubernetes namespace");
        println!("  -s, --selector <selector>       Use a selector for the output JSON");
        println!("  -k, --keys <key1 key2 ...>      Extract specific keys from the output JSON");
        println!("  -f, --filter <key> <value>      Apply filters to the output JSON");
        println!("  --separator                    Print a separator between outputs");
        println!("  --padding                      Add padding (extra lines) between outputs");
        println!("  -h, --help                     Display this help message and exit");
    }
}

fn find_value<'a>(json: &'a Value, keys: &[&str]) -> Option<&'a Value> {
    keys.iter()
        .fold(Some(json), |current_value, &key| match current_value {
            Some(Value::Object(map)) => map.get(key),
            Some(Value::Array(array)) => array.iter().find_map(|item| find_value(item, &[key])),
            _ => None,
        })
}

fn apply_filters(json: &Value, filters: &[(String, String)]) -> bool {
    filters.iter().all(|(key, value)| {
        let keys: Vec<&str> = key.split('.').collect();
        find_value(json, &keys).map_or(false, |found_value| found_value == value)
    })
}

fn format_output(json: &Value, keys: &[String], selector: &Option<String>) -> Option<String> {
    let filtered_json = if keys.is_empty() {
        if let Some(selector) = &selector {
            if let Some(value) = find_value(&json, &selector.split('.').collect::<Vec<&str>>()) {
                value.clone()
            } else {
                Value::Null
            }
        } else {
            json.clone()
        }
    } else {
        let mut filtered_map = Map::new();
        for key in keys {
            if let Some(value) = find_value(json, &key.split('.').collect::<Vec<&str>>()) {
                filtered_map.insert(key.clone(), value.clone());
            }
        }

        filtered_map
            .is_empty()
            .then(|| Value::Null)
            .unwrap_or_else(|| Value::Object(filtered_map))
    };

    if filtered_json.is_null() {
        return None;
    }

    if filtered_json.is_string() {
        return Some(filtered_json.as_str().unwrap().to_string());
    }

    Some(
        serde_json::to_string_pretty(&filtered_json)
            .unwrap()
            .to_colored_json_auto()
            .unwrap(),
    )
}

fn print_separator() {
    let width = term_size::dimensions().map(|(w, _)| w).unwrap_or(80);
    let separator = "=".repeat(width).blue();
    println!("{}", separator);
}

fn print_output(output: &String, args: &Args) {
    if args.separator {
        print_separator();
        if args.padding {
            println!()
        }
    }

    println!("{}", output);
    if args.padding {
        println!()
    }
}

fn execute_stern_command(args: &Args) -> Child {
    if args.selector.is_some() && !args.keys.is_empty() {
        panic!(
            "{}",
            "Cannot use selector (-s) and keys (-k) at the same time".red()
        );
    }

    let mut command = Command::new("stern");
    command.arg(&args.pod_query).arg("-o").arg("raw");

    if let Some(namespace) = &args.namespace {
        command.arg("-n").arg(namespace);
    }

    return command
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to execute stern command");
}

fn main() {
    let (tx, rx) = mpsc::channel();
    ctrlc::set_handler(move || {
        tx.send(()).expect("Could not send signal on channel.");
    })
    .expect("Error setting Ctrl-C handler");

    let args = Args::new(std::env::args().collect());
    let mut child = execute_stern_command(&args);

    let stdout = child.stdout.take().expect("failed to get stdout");

    let filters = args.filters.clone();
    let keys = args.keys.clone();
    let selector = args.selector.clone();

    let handle = thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            match line {
                Ok(line) => match serde_json::from_str::<Value>(&line) {
                    Ok(json) => {
                        if apply_filters(&json, &filters) {
                            let output = format_output(&json, &keys, &selector);
                            if output.is_some() {
                                print_output(&output.unwrap(), &args);
                            }
                        }
                    }
                    Err(_) => println!("{}", line),
                },
                Err(err) => eprintln!("error reading line: {}", err),
            }
        }
    });

    rx.recv().expect("Could not receive from channel.");
    child.kill().expect("Failed to kill child process");
    handle.join().expect("Failed to join thread");
}
