/*
    File utility functions
*/

use std::fs::{File, OpenOptions};
use std::io::{self, prelude::*, BufReader, Result};

/*
    File handling
*/

// Run a closure for each line in a file
fn for_each_line_do<F>(filepath: &str, mut closure: F) -> Result<()>
where
    F: FnMut(usize, &str) -> Result<()>, // line number, line
{
    let file = File::open(filepath)?;
    let reader = BufReader::new(file);
    for (line_number, line) in reader.lines().enumerate() {
        closure(line_number, &line.unwrap())?;
    }
    Result::Ok(())
}

// From a file create a new one where each line is replaced using a given function
pub fn replace_lines_in_file<F>(
    in_filepath: &str,
    out_filepath: &str,
    closure: F,
) -> Result<()>
where
    F: Fn(usize, &str) -> String,
{
    let mut out_file =
        OpenOptions::new().create(true).write(true).open(out_filepath)?;
    for_each_line_do(in_filepath, move |line_number, line| {
        writeln!(out_file, "{}", closure(line_number, line))
    })
}

// Find a line in filepath equal to text and return the line number.
// Otherwise, return an error.
// Warning: line numbering starts from 0!
pub fn match_line_in_file(text: &str, filepath: &str) -> Result<usize> {
    let file = File::open(filepath)?;
    let reader = BufReader::new(file);
    for (line_number, line) in reader.lines().enumerate() {
        if line.unwrap() == text {
            return Result::Ok(line_number);
        }
    }
    Result::Err(io::Error::new(
        io::ErrorKind::Other,
        format!("text {} not found in file {}", text, filepath),
    ))
}

pub fn first_line_in_file(filepath: &str) -> String {
    let file = File::open(filepath).unwrap_or_else(|err| {
        panic!("couldn't open file {}: {}", filepath, err);
    });
    let reader = BufReader::new(file);
    let line = reader.lines().next().unwrap_or_else(|| {
        panic!("couldn't get line bc the file had no lines? {}", filepath);
    });
    line.unwrap_or_else(|err| {
        panic!("getting first line in file failed: {}! {}", filepath, err);
    })
}
