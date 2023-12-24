use std::fs::File;
use std::io::{self, BufRead, Read};
use std::time::Instant;
use regex::Regex;
use clap::{Arg, Command};
use walkdir::WalkDir;
use rayon::prelude::*;

fn is_binary_file(file: &File) -> io::Result<bool> {
    let mut buffer = [0; 1024];
    let num_read = file.take(1024).read(&mut buffer)?;
    if num_read == 0 {
        return Ok(false);  // Empty file
    }
    Ok(buffer[..num_read].contains(&0))
}

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
        .arg(Arg::new("include_binary")
            .short('b')
            .long("bin")
            .help("Include binary files in the search"))
        .arg(Arg::new("depth")
            .short('d')
            .long("depth")
            .value_name("DEPTH")
            .help("Set the maximum search depth"))
        .get_matches();

    let content_pattern = matches.get_one::<String>("content_pattern").unwrap();
    let path = matches.get_one::<String>("path").unwrap();
    let filename_pattern = matches.get_one::<String>("filename_pattern").unwrap();
    let depth: Option<usize> = matches
        .get_one::<String>("depth")
        .and_then(|d| d.parse().ok());
    let include_binary = matches.contains_id("include_binary");

    let content_re = Regex::new(content_pattern).unwrap();
    let filename_re = Regex::new(filename_pattern).unwrap();

    // Debug print to confirm input arguments
    println!("Content pattern: {}", content_pattern);
    println!("Filename pattern: {}", filename_pattern);
    println!("Path: {}", path);
    println!("Depth: {:?}", depth);
    println!("Include binary: {}", include_binary);

    let start = Instant::now();

    let walker = WalkDir::new(path)
        .max_depth(depth.unwrap_or(usize::MAX))
        .into_iter()
        .filter_entry(|e| {
            let match_ = filename_re.is_match(e.file_name().to_string_lossy().as_ref());
            println!("Filename check: {}, Match: {}", e.file_name().to_string_lossy(), match_); // Debug print
            match_
        });

    walker
        .filter_map(Result::ok)
        .par_bridge()
        .for_each(|entry| {
            let path = entry.path();
            if path.is_file() {
                println!("Checking file: {}", path.display()); // Debug print

                if let Ok(file) = File::open(path) {
                    if !include_binary && is_binary_file(&file).unwrap_or(false) {
                        return;
                    }

                    let reader = io::BufReader::new(file);
                    for line in reader.lines().filter_map(|l| l.ok()) {
                        if content_re.is_match(&line) {
                            println!("Match found in {}: {}", path.display(), line);
                        }
                    }
                }
            }
        });

    let duration = start.elapsed();
    println!("Search completed in: {:?}", duration);

    Ok(())
}

