use clap::{Arg, ArgAction, Command};
use flexi_logger::{detailed_format, Duplicate, FileSpec, Logger};
use log::{error, warn};
use owo_colors::colored::*;
use regex::Regex;

use std::{
    fs,
    io::{self, BufRead},
    path::{Path, PathBuf},
    process,
};

fn main() {
    // handle Ctrl+C
    ctrlc::set_handler(move || {
        println!(
            "{} {} {} {}",
            "Received Ctrl-C!".bold().red(),
            "🤬",
            "Exit program!".bold().red(),
            "☠",
        );
        process::exit(0)
    })
    .expect("Error setting Ctrl-C handler");

    // get config dir
    let config_dir = check_create_config_dir().unwrap_or_else(|err| {
        error!("Unable to find or create a config directory: {err}");
        process::exit(1);
    });

    // initialize the logger
    let _logger = Logger::try_with_str("info") // log warn and error
        .unwrap()
        .format_for_files(detailed_format) // use timestamp for every log
        .log_to_file(
            FileSpec::default()
                .directory(&config_dir)
                .suppress_timestamp(),
        ) // change directory for logs, no timestamps in the filename
        .append() // use only one logfile
        .duplicate_to_stderr(Duplicate::Info) // print infos, warnings and errors also to the console
        .start()
        .unwrap();

    // FIXME handle emtpy pipes
    // read input from pipe
    let input = read_pipe();
    println!("INPUT: {}", input);

    // handle arguments
    let matches = manipulate_pipe().get_matches();
    let string_flag = matches.get_flag("string");

    if let Some(args) = matches
        .get_many::<String>("args")
        .map(|a| a.collect::<Vec<_>>())
    {
        let mut num_flag: u32 = 0;
        if let Some(n) = matches.get_one::<String>("num") {
            match n.parse::<u32>() {
                Ok(num) => num_flag = num,
                Err(err) => {
                    warn!("Expected an integer for the number of matching patterns: {err}");
                    process::exit(1);
                }
            }
        }

        if string_flag {
            let old_pattern = String::from(args[0]);
            let new_pattern = String::from(args[1]);

            // replace old pattern with new pattern
            let output = find_replace(input, old_pattern, new_pattern, num_flag);
            println!("OUTPUT: {}", output);
        } else {
            let re = Regex::new(args[0]).unwrap();
            let new_pattern = args[1].as_str();

            let output = find_replace_regex(input, re, new_pattern, num_flag);
            println!("OUTPUT: {}", output);
        }
    } else {
        // handle commands
        match matches.subcommand() {
            Some(("log", _)) => {
                if let Ok(logs) = show_log_file(&config_dir) {
                    println!("{}", "Available logs:".bold().yellow());
                    println!("{}", logs);
                } else {
                    error!("Unable to read logs");
                    process::exit(1);
                }
            }
            _ => {
                unreachable!();
            }
        }
    }
}

// build cli
fn manipulate_pipe() -> Command {
    Command::new("map")
        .bin_name("wf")
        .before_help(format!(
            "{}\n{}",
            "MAP".bold().truecolor(250, 0, 104),
            "Leann Phydon <leann.phydon@gmail.com>".italic().dimmed()
        ))
        .about("Manipulate pipes")
        .before_long_help(format!(
            "{}\n{}",
            "MAP".bold().truecolor(250, 0, 104),
            "Leann Phydon <leann.phydon@gmail.com>".italic().dimmed()
        ))
        .long_about(format!("{}", "Manipulate pipes",))
        // TODO update version
        .version("1.0.0")
        .author("Leann Phydon <leann.phydon@gmail.com>")
        .arg_required_else_help(true)
        .arg(
            Arg::new("args")
                .help("Replace the old pattern with the new pattern")
                .action(ArgAction::Set)
                .num_args(2)
                .value_names(["OLD_PATTERN", "NEW_PATTERN"]),
        )
        .arg(
            Arg::new("num")
                .short('n')
                .long("num")
                .help("Replaces first N matches of a pattern with another string")
                .long_help(format!("{}\n{}", "Replaces first N matches of a pattern with another string", "If N is negativ, it replaces in reversed order, starting from the last matching pattern"))
                .action(ArgAction::Set)
                .num_args(1)
                .value_name("NUMBER")
        )
        .arg(
            Arg::new("string")
                .short('s')
                .long("string")
                .help("Treat the pattern as a literal string")
                .action(ArgAction::SetTrue),
        )
        .subcommand(
            Command::new("log")
                .short_flag('L')
                .long_flag("log")
                .about("Show content of the log file"),
        )
}

fn read_pipe() -> String {
    // FIXME handle emtpy stdin when locked
    let input = io::stdin()
        // TODO lock here?
        .lock()
        .lines()
        .fold("".to_string(), |acc, line| acc + &line.unwrap());

    input
}

fn find_replace(input: String, old_pattern: String, new_pattern: String, num_flag: u32) -> String {
    if num_flag == 0 {
        input.replace(&old_pattern, &new_pattern)
    } else {
        input.replacen(&old_pattern, &new_pattern, num_flag as usize)
    }
}

fn find_replace_regex(input: String, re: Regex, new_pattern: &str, num_flag: u32) -> String {
    let result = re.replacen(&input, num_flag as usize, new_pattern);
    result.to_string()
}

fn check_create_config_dir() -> io::Result<PathBuf> {
    let mut new_dir = PathBuf::new();
    match dirs::config_dir() {
        Some(config_dir) => {
            new_dir.push(config_dir);
            new_dir.push("map");
            if !new_dir.as_path().exists() {
                fs::create_dir(&new_dir)?;
            }
        }
        None => {
            error!("Unable to find config directory");
        }
    }

    Ok(new_dir)
}

fn show_log_file(config_dir: &PathBuf) -> io::Result<String> {
    let log_path = Path::new(&config_dir).join("map.log");
    match log_path.try_exists()? {
        true => {
            return Ok(format!(
                "{} {}\n{}",
                "Log location:".italic().dimmed(),
                &log_path.display(),
                fs::read_to_string(&log_path)?
            ));
        }
        false => {
            return Ok(format!(
                "{} {}",
                "No log file found:"
                    .truecolor(250, 0, 104)
                    .bold()
                    .to_string(),
                log_path.display()
            ))
        }
    }
}
