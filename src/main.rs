use std::io::{self, BufRead};

fn main() {
    // FIXME handle emtpy stdin whe  locked
    let input = io::stdin()
        .lock()
        .lines()
        .fold("".to_string(), |acc, line| acc + &line.unwrap() + "\n");

    if input.is_empty() {
        println!("No Input");
    } else {
        println!("{}", input);
    }
}
