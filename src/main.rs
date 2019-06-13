use std::cmp;
use std::fmt::{self, Display, Formatter};
use std::io::{self, BufRead};
use std::str::FromStr;

use console::style;
use regex::Regex;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "rust-errors")]
struct Opt {
    #[structopt(short = "l", long = "lines", default_value = "5")]
    lines: u32,
}

#[derive(Debug)]
struct Error {
    num: String,
    message: String,
    file: String,
    line: u32,
    column: u32,
    details: String,
}

impl Error {
    fn new(num: &str, message: &str, file: &str, line: u32, column: u32, details: &str) -> Self {
        Error {
            num: num.to_owned(),
            message: message.to_owned(),
            file: file.to_owned(),
            line,
            column,
            details: details.to_owned(),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}: {}\n   {} {}:{}\n\n{}",
            style(format!("E{}", self.num)).underlined().red(),
            style(&self.message).yellow(),
            style(&self.file).italic().bold(),
            style(self.line).cyan(),
            style(self.column).cyan(),
            style(&self.details).dim()
        )
    }
}

impl FromStr for Error {
    type Err = String;

    fn from_str(inp: &str) -> Result<Self, Self::Err> {
        // split input into 3 lines delimited by \n
        let lines: Vec<&str> = inp.splitn(3, '\n').collect();

        // extract error number and message
        let err = Regex::new(r"error\[E([0-9]+)\]: (.*)")
            .unwrap()
            .captures(lines[0])
            .unwrap();
        let err_num = err.get(1).unwrap();
        let err_msg = err.get(2).unwrap();

        // extract file, line and col
        let context = Regex::new(r" +--> ([^:]+):([0-9]+):([0-9]+)")
            .unwrap()
            .captures(lines[1])
            .unwrap();
        let file = context.get(1).unwrap();
        let line = context.get(2).unwrap();
        let col = context.get(3).unwrap();

        Ok(Error::new(
            err_num.as_str(),
            err_msg.as_str(),
            file.as_str(),
            line.as_str().parse().unwrap(),
            col.as_str().parse().unwrap(),
            lines[2],
        ))
    }
}

fn main() {
    let opt = Opt::from_args();
    let errors = read_errors();

    for i in 0..cmp::min(opt.lines as usize, errors.len()) {
        print_line("┄");
        print!("{}", errors[i]);
    }

    print!("\n");
    print_line("░");
}

fn print_line(ch: &str) {
    let line_length = 100;
    for _ in 0..line_length {
        print!("{}", ch);
    }
    print!("\n");
}

fn read_errors() -> Vec<Error> {
    let stdin = io::stdin();
    let re = Regex::new(r"error\[E([0-9])+\]:.*").unwrap();
    let mut errors = vec![];
    let mut current_error = String::new();
    let mut reading_error = false;
    for line in stdin.lock().lines().map(|l| l.unwrap()) {
        if re.is_match(&line) {
            if !current_error.is_empty() {
                errors.push(current_error.parse().unwrap());
            }

            if !reading_error {
                reading_error = true;
            }

            current_error = format!("{}\n", line);
        } else {
            if reading_error {
                if line.is_empty() == false {
                    current_error.push_str(&format!("{}\n", line));
                } else {
                    reading_error = false;
                }
            }
        }
    }

    errors
}
