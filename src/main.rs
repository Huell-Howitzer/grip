use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use regex::Regex;
use std::time::Instant;
use clap::{Arg, Command, ArgAction};
use walkdir::WalkDir;
use rayon::prelude::*;

fn is_binary_file(file: &File) -> io::Result<bool> {
    let mut buffer = [0; 1024];
    let mut limited_reader = file.take(1024);
    let num_read = limited_reader.read(&mut buffer)?;
    if num_read == 0 {
        return Ok(false);
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
        .long("include-binary")
        .action(ArgAction::SetFalse)
        .help("Include binary files in the search"))
      .get_matches();

    let content_pattern = matches.get_one::<String>("content_pattern").unwrap();
    let path = matches.get_one::<String>("path").unwrap();
    let filename_pattern = matches.get_one::<String>("filename_pattern").unwrap();
    let include_binary = matches.get_flag("include_binary");

    let content_re = Regex::new(content_pattern).unwrap();
    let filename_re = Regex::new(filename_pattern).unwrap();

    let start = Instant::now();

    WalkDir::new(path)
      .into_iter()
      .filter_map(Result::ok)
      .par_bridge() // Use rayon's parallel bridge
      .for_each(|entry| {
          let path = entry.path();
          if path.is_file() && filename_re.is_match(path.to_string_lossy().as_ref()) {
              if let Ok(file) = File::open(path) {
                  if !include_binary && is_binary_file(&file).unwrap_or(false) {
                      return;
                  }
                  let reader = BufReader::new(file);
                  for line in reader.lines().filter_map(Result::ok) {
                      let mut highlighted_line = String::new();
                      let mut last_end = 0;

                      for mat in content_re.find_iter(&line) {
                          // Append the non-matching part
                          highlighted_line.push_str(&line[last_end..mat.start()]);
                          // Append the matching part, highlighted
                          highlighted_line.push_str(&format!("\x1b[31m{}\x1b[0m", &line[mat.start()..mat.end()]));
                          last_end = mat.end();
                      }

                      // Append the rest of the line
                      highlighted_line.push_str(&line[last_end..]);

                      if last_end > 0 { // If there was a match
                          println!("{}: {}", path.display(), highlighted_line);
                      }
                  }
              }
          }
      });

    let duration = start.elapsed();
    println!("Elapsed time: {:?}", duration);

    Ok(())
}


