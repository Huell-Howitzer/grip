use std::fs::File;
use std::{fs, io::{self, BufRead}};
use regex::Regex;
use std::time::Instant;
use clap::{Arg, Command};
use walkdir::WalkDir;
use rayon::prelude::*;

fn main() -> io::Result<()> {
    let matches = Command::new("grip")
        .version("0.1.0")
        .author("Ryan M. Howell")
        .about("Global Regex Inspector and Printer")
        .arg(Arg::new("content_pattern")
            .short('p')
            .long("pattern")
            .value_name("CONTENT_PATTERN")
            .help("The Regex pattern to search for in the content")
            .required(true))
        .arg(Arg::new("path")
            .help("The path to search")
            .required(true)
            .index(1))
        .arg(Arg::new("filename_pattern")
            .short('f')
            .long("filename")
            .value_name("FILENAME_PATTERN")
            .help("Regex pattern to match filenames")
            .required(true))
        .get_matches();

    let content_pattern = matches.get_one::<String>("content_pattern").unwrap();
    let path = matches.get_one::<String>("path").unwrap();
    let filename_pattern = matches.get_one::<String>("filename_pattern").unwrap();

    let content_re = Regex::new(content_pattern).unwrap();
    let filename_re = Regex::new(filename_pattern).unwrap();

    let start = Instant::now();

    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .par_bridge() // Use rayon's parallel bridge
        .for_each(|entry| {
            let path = entry.path();
            if path.is_file() && filename_re.is_match(path.to_string_lossy().as_ref()) {
                if let Ok(file) = File::open(path) {
                    let reader = io::BufReader::new(file);
                    for line in reader.lines().filter_map(|l| l.ok()) {
                        if content_re.is_match(&line) {
                            println!("{}: {}", path.display(), line);
                        }
                    }
                }
            }
        });

    let duration = start.elapsed();

    println!("Elapsed time: {:?}", duration);

    Ok(())
}

