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

    // handle arguments
    let matches = manipulate_pipe().get_matches();
    let string_flag = matches.get_flag("string");

    if let Some(subcmd) = matches.subcommand_matches("cut") {
        if let Some(arg) = subcmd.get_one::<String>("subarg") {
            // read input from pipe
            let input = read_pipe();

            let output = cut(input, arg.to_owned());
            println!("{}", output);
        }
    } else if let Some(args) = matches
        .get_many::<String>("args")
        .map(|a| a.collect::<Vec<_>>())
    {
        let mut num_flag: u32 = 0;
        if let Some(n) = matches.get_one::<String>("num") {
            match n.parse::<u32>() {
                Ok(num) => num_flag = num,
                Err(err) => {
                    warn!("Expected an integer for the number of matching patterns: {err}");
                    process::exit(0);
                }
            }
        }

        // read input from pipe
        let input = read_pipe();

        if string_flag {
            // Treat the pattern as a literal string
            let old_pattern = String::from(args[0]);
            let new_pattern = String::from(args[1]);

            // replace old pattern with new pattern
            let output = find_replace_string(input, old_pattern, new_pattern, num_flag);
            println!("{}", output);
        } else {
            // Treat the pattern as a regex
            let re = Regex::new(args[0]).unwrap();
            let new_pattern = args[1].as_str();

            let output = find_replace_regex(input, re, new_pattern, num_flag);
            println!("{}", output);
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
            Some(("syntax", _)) => {
                show_regex_syntax();
            }
            _ => {
                // FIXME it is reachable -> still?
                unreachable!();
            }
        }
    }
}

// build cli
fn manipulate_pipe() -> Command {
    Command::new("map")
        .bin_name("map")
        .before_help(format!(
            "{}\n{}",
            "MAP".bold().truecolor(250, 0, 104),
            "Leann Phydon <leann.phydon@gmail.com>".italic().dimmed()
        ))
        .about("MAnipulate Pipes")
        .before_long_help(format!(
            "{}\n{}",
            "MAP".bold().truecolor(250, 0, 104),
            "Leann Phydon <leann.phydon@gmail.com>".italic().dimmed()
        ))
        .long_about(format!(
            "{}\n\n- {}\n\t{}",
            "MAnipulate Pipes", "Regex syntax:", "https://docs.rs/regex/latest/regex/#syntax"
        ))
        // TODO update version
        .version("1.2.1")
        .author("Leann Phydon <leann.phydon@gmail.com>")
        .arg_required_else_help(true)
        .arg(
            Arg::new("args")
                .help("Replace a pattern with a new one")
                .action(ArgAction::Set)
                .num_args(2)
                .value_names(["OLD_PATTERN", "NEW_PATTERN"]),
        )
        .arg(
            Arg::new("num")
                .short('n')
                .long("num")
                .help("Replaces first N matches of a pattern with another string")
                .action(ArgAction::Set)
                .num_args(1)
                .value_name("NUMBER"),
        )
        .arg(
            Arg::new("string")
                .short('s')
                .long("string")
                .help("Treat the pattern as a literal string")
                .action(ArgAction::SetTrue),
        )
        .subcommand(
            Command::new("cut")
                .short_flag('c')
                .long_flag("cut")
                .about("Select relevant parts from the input")
                .long_about(format!(
                    "{}\n{}\n{}",
                    "Select relevant parts from the input",
                    "Choose words by there indices",
                    "Indices must be space separated and start from '0'"
                ))
                .arg_required_else_help(true)
                .arg(
                    Arg::new("subarg")
                        .help("Space separated indices of relevant selections")
                        .long_help(format!(
                            "{}\n{}\n{}",
                            "Space separated indices of relevant selections",
                            "For example: '0 3 4'",
                            "Selects the first, the third and the forth word from a given input"
                        ))
                        .action(ArgAction::Set)
                        .num_args(1)
                        .value_name("SELECTIONS"),
                ),
        )
        .subcommand(
            Command::new("log")
                .short_flag('L')
                .long_flag("log")
                .about("Show content of the log file"),
        )
        .subcommand(
            Command::new("syntax")
                .short_flag('S')
                .long_flag("syntax")
                .about("Show regex syntax information"),
        )
}

fn read_pipe() -> String {
    let mut input = io::stdin()
        .lock()
        .lines()
        .fold("".to_string(), |acc, line| acc + &line.unwrap() + "\n");

    // TODO possible error here?
    // TODO if last char is '\n' it will get removed
    let _ = input.pop();

    input.trim().to_string()
}

fn cut(input: String, selection: String) -> String {
    if selection.is_empty() {
        return input;
    }

    let selector = parse_selection(selection);
    let splitted_input: Vec<&str> = input.split_ascii_whitespace().collect();

    let max_selection = *selector.iter().max().unwrap();
    let len_input = splitted_input.len() as u32;

    if len_input <= max_selection {
        warn!(
            "{}",
            format!(
                "Selection out of range\nSelection = '{}'\nLength of input = '{}'\nIndices start from '0' => Max Selection = '{}'",
                max_selection, len_input, len_input - 1
            )
        );
        process::exit(0);
    }

    let mut cut_string: Vec<&str> = Vec::new();

    for i in selector {
        cut_string.push(splitted_input[i as usize]);
    }

    cut_string.join(" ")
}

fn parse_selection(selection: String) -> Vec<u32> {
    let selector: Vec<u32> = selection
        .split_ascii_whitespace()
        .map(|it| {
            it.parse::<u32>().unwrap_or_else(|err| {
                warn!("Argument must be of type u32: {err}");
                process::exit(0);
            })
        })
        .collect();

    selector
}

fn find_replace_string(
    input: String,
    old_pattern: String,
    new_pattern: String,
    num_flag: u32,
) -> String {
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

fn show_regex_syntax() {
    println!("{}", "Regex Syntax".bold().blue());
    println!(
        "More information on '{}'",
        "https://docs.rs/regex/latest/regex/#syntax".italic()
    );
    println!("\n{}", "Matching one character:".bold());
    println!(
        r###"
.             any character except new line (includes new line with s flag)
[0-9]         any ASCII digit
\d            digit (\p{{Nd}})
\D            not digit
\pX           Unicode character class identified by a one-letter name
\p{{Greek}}     Unicode character class (general category or script)
\PX           Negated Unicode character class identified by a one-letter name
\P{{Greek}}     negated Unicode character class (general category or script)
        "###
    );
    println!("\n{}", "Character classes:".bold());
    println!(
        r###"
[xyz]         A character class matching either x, y or z (union).
[^xyz]        A character class matching any character except x, y and z.
[a-z]         A character class matching any character in range a-z.
[[:alpha:]]   ASCII character class ([A-Za-z])
[[:^alpha:]]  Negated ASCII character class ([^A-Za-z])
[x[^xyz]]     Nested/grouping character class (matching any character except y and z)
[a-y&&xyz]    Intersection (matching x or y)
[0-9&&[^4]]   Subtraction using intersection and negation (matching 0-9 except 4)
[0-9--4]      Direct subtraction (matching 0-9 except 4)
[a-g~~b-h]    Symmetric difference (matching `a` and `h` only)
[\[\]]        Escaping in character classes (matching [ or ])
[a&&b]        An empty character class matching nothing        
        "###
    );
    println!("\n{}", "Repetitions:".bold());
    println!(
        r###"
x*        zero or more of x (greedy)
x+        one or more of x (greedy)
x?        zero or one of x (greedy)
x*?       zero or more of x (ungreedy/lazy)
x+?       one or more of x (ungreedy/lazy)
x??       zero or one of x (ungreedy/lazy)
x{{n,m}}    at least n x and at most m x (greedy)
x{{n,}}     at least n x (greedy)
x{{n}}      exactly n x
x{{n,m}}?   at least n x and at most m x (ungreedy/lazy)
x{{n,}}?    at least n x (ungreedy/lazy)
x{{n}}?     exactly n x        
        "###
    );
    println!("\n{}", "Empty matches:".bold());
    println!(
        r###"
^               the beginning of a haystack (or start-of-line with multi-line mode)
$               the end of a haystack (or end-of-line with multi-line mode)
\A              only the beginning of a haystack (even with multi-line mode enabled)
\z              only the end of a haystack (even with multi-line mode enabled)
\b              a Unicode word boundary (\w on one side and \W, \A, or \z on other)
\B              not a Unicode word boundary
\b{{start}}, \<   a Unicode start-of-word boundary (\W|\A on the left, \w on the right)
\b{{end}}, \>     a Unicode end-of-word boundary (\w on the left, \W|\z on the right))
\b{{start-half}}  half of a Unicode start-of-word boundary (\W|\A on the left)
\b{{end-half}}    half of a Unicode end-of-word boundary (\W|\z on the right)        
        "###
    );
    println!("\n{}", "Grouping and flags:".bold());
    println!(
        r###"
(exp)          numbered capture group (indexed by opening parenthesis)
(?P<name>exp)  named (also numbered) capture group (names must be alpha-numeric)
(?<name>exp)   named (also numbered) capture group (names must be alpha-numeric)
(?:exp)        non-capturing group
(?flags)       set flags within current group
(?flags:exp)   set flags for exp (non-capturing)        
        "###
    );
    println!("\n{}", "Flags:".bold());
    println!(
        r###"
i     case-insensitive: letters match both upper and lower case
m     multi-line mode: ^ and $ match begin/end of line
s     allow . to match \n
R     enables CRLF mode: when multi-line mode is enabled, \r\n is used
U     swap the meaning of x* and x*?
u     Unicode support (enabled by default)
x     verbose mode, ignores whitespace and allow line comments (starting with `#`)        
        "###
    );
    println!("\n{}", "Escape sequences:".bold());
    println!(
        r###"
\*              literal *, applies to all ASCII except [0-9A-Za-z<>]
\a              bell (\x07)
\f              form feed (\x0C)
\t              horizontal tab
\n              new line
\r              carriage return
\v              vertical tab (\x0B)
\A              matches at the beginning of a haystack
\z              matches at the end of a haystack
\b              word boundary assertion
\B              negated word boundary assertion
\b{{start}}, \<   start-of-word boundary assertion
\b{{end}}, \>     end-of-word boundary assertion
\b{{start-half}}  half of a start-of-word boundary assertion
\b{{end-half}}    half of a end-of-word boundary assertion
\123            octal character code, up to three digits (when enabled)
\x7F            hex character code (exactly two digits)
\x{{10FFFF}}      any hex character code corresponding to a Unicode code point
\u007F          hex character code (exactly four digits)
\u{{7F}}          any hex character code corresponding to a Unicode code point
\U0000007F      hex character code (exactly eight digits)
\U{{7F}}          any hex character code corresponding to a Unicode code point
\p{{Letter}}      Unicode character class
\P{{Letter}}      negated Unicode character class
\d, \s, \w      Perl character class
\D, \S, \W      negated Perl character class        
        "###
    );
    println!("\n{}", "Perl character classes:".bold());
    println!(
        r###"
\d     digit (\p{{Nd}})
\D     not digit
\s     whitespace (\p{{White_Space}})
\S     not whitespace
\w     word character (\p{{Alphabetic}} + \p{{M}} + \d + \p{{Pc}} + \p{{Join_Control}})
\W     not word character        
        "###
    );
    println!("\n{}", "ASCII character classes:".bold());
    println!(
        r###"
[[:alnum:]]    alphanumeric ([0-9A-Za-z])
[[:alpha:]]    alphabetic ([A-Za-z])
[[:ascii:]]    ASCII ([\x00-\x7F])
[[:blank:]]    blank ([\t ])
[[:cntrl:]]    control ([\x00-\x1F\x7F])
[[:digit:]]    digits ([0-9])
[[:graph:]]    graphical ([!-~])
[[:lower:]]    lower case ([a-z])
[[:print:]]    printable ([ -~])
[[:punct:]]    punctuation ([!-/:-@\[-`{{}}-~])
[[:space:]]    whitespace ([\t\n\v\f\r ])
[[:upper:]]    upper case ([A-Z])
[[:word:]]     word characters ([0-9A-Za-z_])
[[:xdigit:]]   hex digit ([0-9A-Fa-f])        
        "###
    );
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
