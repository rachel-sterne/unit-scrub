use std::env;
use std::fs;
use std::io::{self, Read};
use std::process::ExitCode;

use unit_scrub::parse_document;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();

    let input = match args.first() {
        Some(path) => match fs::read_to_string(path) {
            Ok(text) => text,
            Err(err) => {
                eprintln!("could not read {}: {}", path, err);
                return ExitCode::FAILURE;
            }
        },
        None => {
            let mut buf = String::new();
            if let Err(err) = io::stdin().read_to_string(&mut buf) {
                eprintln!("could not read stdin: {}", err);
                return ExitCode::FAILURE;
            }
            buf
        }
    };

    let results = parse_document(&input);
    let mut had_error = false;

    for result in results {
        match result {
            Ok(size) => println!("{}", size),
            Err(err) => {
                eprintln!("{}", err);
                had_error = true;
            }
        }
    }

    if had_error {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
