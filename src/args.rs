use crate::types::*;
use crate::util::*;

use std::env::args;
use std::fs::metadata;
use std::path::Path;
use std::process::exit;

pub fn get_args_or_exit() -> Arguments {
    let mut args = args();
    let _ = args.next(); // Skip exec path
    let src_path = validate_or_exit(args.next(), true);
    let dst_path = validate_or_exit(args.next(), false);
    if let Some(_) = args.next() {
        // Check for extra args
        print_help_and_exit();
    }

    Arguments { src_path, dst_path }
}

fn validate_or_exit(file_path: Option<String>, exists: bool) -> String {
    if let Some(mut file_path) = file_path {
        if let Some(first_char) = file_path.chars().next() {
            if first_char == '~' {
                file_path = format!("{}{}", home_dir(), file_path[1..].to_string());
            }
        } else {
            print_error_and_exit("Could not find home directory.");
        }
        if exists {
            if let Ok(meta_data) = metadata(file_path.clone()) {
                if meta_data.is_dir() {
                    return file_path;
                }
            }
            print_error_and_exit(&format!("Not a directory: {}.", file_path));
        } else {
            if !Path::new(&file_path).exists() {
                return file_path;
            } else {
                print_error_and_exit(&format!("File already exists at: {}.", file_path));
            }
        }
    }
    print_help_and_exit();
    String::from("")
}

fn print_help_and_exit() {
    println!("");
    println!("A fast replacement for cp -R.");
    println!("");
    println!("Usage: cpr from_dir to_dir");
    println!("");
    println!("Description: Recursively copy 'from_dir' to 'to_dir'. This should be 5-6 times faster than cp -R.");
    println!("");

    exit(-1);
}
