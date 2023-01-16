use console::Term;
use core::panic;
use std::env::args;
use std::fs::{copy, create_dir, metadata, read_dir, read_link};
use std::io::{self, Error, Write};
use std::os::unix::fs::symlink;
use std::path::Path;
use std::process::exit;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread::{self, JoinHandle};

struct CopyInfo {
    pub src_path: String,
    pub dst_path: String,
    pub is_symlink: bool,
    pub size: u64,
}

struct SplitCopyInfo {
    pub size: u64,
    pub copy_info_list: Vec<CopyInfo>,
}

impl SplitCopyInfo {
    pub fn new() -> SplitCopyInfo {
        SplitCopyInfo {
            size: 0,
            copy_info_list: Vec::<CopyInfo>::new(),
        }
    }
}

struct Arguments {
    pub src_path: String,
    pub dst_path: String,
}

fn scan_source_prep_dest(
    src_dir_path: &str,
    dst_dir_path: &str,
    copy_info_list: &mut Vec<CopyInfo>,
) -> Result<(), Error> {
    // Create dir at dest
    create_dir(dst_dir_path)?;

    // Scan src path
    for entry in read_dir(src_dir_path)? {
        let entry = entry?;
        let meta = entry.metadata()?;

        let src_file_path = entry
            .path()
            .as_os_str()
            .to_str()
            .expect("invalid file name")
            .to_string();
        let file_name = entry
            .file_name()
            .to_str()
            .expect("invalid file name")
            .to_string();
        let dst_file_path = format!("{}/{}", dst_dir_path, file_name);

        if meta.is_dir() {
            // Recurse thru dirs
            scan_source_prep_dest(&src_file_path, &dst_file_path, copy_info_list)?;
        } else {
            // Memoize files
            copy_info_list.push(CopyInfo {
                src_path: src_file_path.to_string(),
                dst_path: dst_file_path,
                is_symlink: meta.is_symlink(),
                size: meta.len(),
            });
        }
    }

    Ok(())
}

// Given one copy_info_list, split it into n split_copy_info structs where each
// one has roughly the same number of bytes in it.
fn balanced_split(copy_info_list: Vec<CopyInfo>, n: i32) -> Vec<SplitCopyInfo> {
    // Create the output struct
    let mut split_copy_info_list = Vec::<SplitCopyInfo>::new();
    for _ in 0..n {
        split_copy_info_list.push(SplitCopyInfo::new());
    }

    for file_info in copy_info_list {
        // find the bucket with the least bytes in it
        let (mut index, mut min_size) = (0 as u64, u64::MAX);
        let mut i = 0;
        for file_info in &split_copy_info_list {
            if file_info.size < min_size {
                (index, min_size) = (i, file_info.size);
            }
            i += 1;
        }

        // Put file_info into that bucket
        let split_file_info = split_copy_info_list
            .get_mut(index as usize)
            .expect("index out of bounds.");
        split_file_info.size += file_info.size;
        split_file_info.copy_info_list.push(file_info);
    }

    split_copy_info_list
}

fn output_file_error(file_path: &str, error: Error) {
    let term = Term::stdout();
    term.clear_line();
    println!("Copy Link Error: {}", file_path);
    println!("            {}", error);
}

fn copy_file_list(split_file_info: SplitCopyInfo, sender: Sender<CopyInfo>) {
    for copy_info in split_file_info.copy_info_list {
        if !copy_info.is_symlink {
            // File
            if let Err(err) = copy(&copy_info.src_path, &copy_info.dst_path) {
                output_file_error(&copy_info.src_path, err);
            }
        } else {
            // SymLink
            match read_link(&copy_info.src_path) {
                Ok(original) => {
                    if let Err(err) = symlink(&original, &copy_info.dst_path) {
                        output_file_error(&copy_info.src_path, err);
                    }
                }
                Err(err) => {
                    output_file_error(&copy_info.src_path, err);
                }
            }
        }
        sender.send(copy_info);
    }
}

fn bytes_to_human_readable(bytes: u64) -> String {
    let mut bytes = bytes as f64;
    let mut units = "bytes";

    if bytes > 1000.0 {
        bytes /= 1024.0;
        units = "KB";
    }
    if bytes > 1000.0 {
        bytes /= 1024.0;
        units = "MB";
    }
    if bytes > 1000.0 {
        bytes /= 1024.0;
        units = "GB";
    }

    return format!("{:.3} {}", bytes, units);
}

fn bytes_in_file_list(copy_info_list: &Vec<CopyInfo>) -> u64 {
    let mut byte_count = 0;
    for copy_info in copy_info_list {
        byte_count += copy_info.size;
    }

    byte_count
}

fn home_dir() -> String {
    let home_dir = dirs::home_dir().expect("Could not find home dir.");
    let home_dir = home_dir
        .as_os_str()
        .to_str()
        .expect("Could not convert home dir to utf8 string.");
    home_dir.to_string()
}
fn validate(file_path: Option<String>, exists: bool) -> String {
    if let Some(mut file_path) = file_path {
        if let Some(first_char) = file_path.chars().next() {
            if first_char == '~' {
                file_path = format!("{}{}", home_dir(), file_path[1..].to_string());
            }
        } else {
            panic!("Could not find home directory.");
        }
        if exists {
            if let Ok(meta_data) = metadata(file_path.clone()) {
                if meta_data.is_dir() {
                    return file_path;
                }
            }
            panic!("Not a directory: {}", file_path);
        } else {
            if !Path::new(&file_path).exists() {
                return file_path;
            } else {
                panic!("File already exists at: {}.", file_path);
            }
        }
    }
    help();
    "".to_string()
}

fn get_args() -> Arguments {
    let mut args = args();
    let _ = args.next();
    let src_path = validate(args.next(), true);
    let dst_path = validate(args.next(), false);
    if let Some(_) = args.next() {
        help();
    }

    Arguments { src_path, dst_path }
}

fn cpr(src_path: String, dst_path: String) {
    let _ = io::stdout().write("...".as_bytes());
    let _ = io::stdout().flush();

    let mut copy_info_list = Vec::<CopyInfo>::new();
    if let Err(err) = scan_source_prep_dest(&src_path, &dst_path, &mut copy_info_list) {
        println!("{}", err);
    }
    let bytes_to_copy = bytes_in_file_list(&copy_info_list) as f64;
    let bytes_copied_str = bytes_to_human_readable(bytes_in_file_list(&copy_info_list));

    let split_file_info_list = balanced_split(copy_info_list, 12);

    let (sender, receiver) = mpsc::channel::<CopyInfo>();
    let mut join_handle_list = Vec::<JoinHandle<()>>::new();
    for split_file_info in split_file_info_list {
        let sender = sender.clone();
        let join_handle = thread::spawn(move || {
            copy_file_list(split_file_info, sender);
        });
        join_handle_list.push(join_handle);
    }
    drop(sender);

    let term = Term::stdout();
    let mut current_bytes_copied = 0;
    while let Ok(copy_info) = receiver.recv() {
        current_bytes_copied += copy_info.size;
        let p = (current_bytes_copied as f64 / bytes_to_copy) * 100.0;
        let p_str = format!("{:.2}", p);
        let _ = term.clear_line();
        let _ = io::stdout().write(format!("{}%", p_str).as_bytes());
        let _ = io::stdout().flush();
    }

    for join_handle in join_handle_list {
        let _ = join_handle.join();
    }

    term.clear_line();
    println!("{} copied.", bytes_copied_str);
}

fn help() {
    println!("");
    println!("cpr - Fast replacement for cp -R");
    println!("");
    println!("Usage: cpr source_dir destination_dir");
    println!("");
    println!("Description: Recursively copy 'source_dir' to 'destination_dir'. This should be 5-6 times faster than cp -R.");
    println!("");
    exit(-1);
}

fn main() {
    let args = get_args();
    cpr(args.src_path, args.dst_path);
}
