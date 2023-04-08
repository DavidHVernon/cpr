use crate::types::*;
use crate::util::*;
use console::Term;
use std::fs::{copy, create_dir, read_dir, read_link};
use std::io::{self, Error, Write};
use std::os::unix::fs::symlink;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread::{self, JoinHandle};

pub fn cpr(args: Arguments) {
    let (sender, receiver) = mpsc::channel::<CopyInfo>();

    let (join_handle_list, bytes_to_copy) = cpr_prepare(args, sender);

    cpr_monitor(join_handle_list, bytes_to_copy, receiver);
}

fn cpr_prepare(args: Arguments, sender: Sender<CopyInfo>) -> (Vec<JoinHandle<()>>, u64) {
    // Let the user know we're getting ready to copy.
    let _ = io::stdout().write("...".as_bytes());
    let _ = io::stdout().flush();

    let copy_info_list = scan_source_prep_dest(&args.src_path, &args.dst_path);
    let bytes_to_copy = bytes_in_file_list(&copy_info_list);

    // Given the list of all the files we need to copy, split them up into chunks.
    let split_file_info_list = balanced_split(copy_info_list, 50);

    // Spawn some threads, give one chunk to each thread.
    let mut join_handle_list = Vec::<JoinHandle<()>>::new();
    for split_file_info in split_file_info_list {
        let sender = sender.clone();
        let join_handle = thread::spawn(move || {
            copy_file_list(split_file_info, sender);
        });
        join_handle_list.push(join_handle);
    }
    drop(sender);

    (join_handle_list, bytes_to_copy)
}

fn cpr_monitor(
    join_handle_list: Vec<JoinHandle<()>>,
    bytes_to_copy: u64,
    receiver: Receiver<CopyInfo>,
) {
    let term = Term::stdout();
    let mut current_bytes_copied = 0;
    // As threads do there things, we get these messages back.
    while let Ok(copy_info) = receiver.recv() {
        current_bytes_copied += copy_info.size;
        let p = (current_bytes_copied as f64 / bytes_to_copy as f64) * 100.0;
        let p_str = format!("{:.2}", p);
        let _ = term.clear_line();
        let _ = io::stdout().write(format!("{}%", p_str).as_bytes());
        let _ = io::stdout().flush();
    }

    // Once all the sender side channels have closed, wait on threads to exit.
    for join_handle in join_handle_list {
        let _ = join_handle.join();
    }

    // Output completion to user.
    let _ = term.clear_line();
    println!("{} copied.", human_readable_bytes(bytes_to_copy));
}

fn scan_source_prep_dest(src_dir_path: &str, dst_dir_path: &str) -> Vec<CopyInfo> {
    let mut copy_info_list = Vec::new();

    if let Err(error) = _scan_source_prep_dest(src_dir_path, dst_dir_path, &mut copy_info_list) {
        output_file_error(src_dir_path, error);
    }

    copy_info_list
}

fn _scan_source_prep_dest(
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

        let src_file_path = entry.path().to_string_lossy().to_string();
        let file_name = entry.file_name().to_string_lossy().to_string();
        let dst_file_path = format!("{}/{}", dst_dir_path, file_name);

        if meta.is_dir() {
            // Recurse thru dirs
            _scan_source_prep_dest(&src_file_path, &dst_file_path, copy_info_list)?;
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
        let _ = sender.send(copy_info);
    }
}

fn output_file_error(file_path: &str, error: Error) {
    let term = Term::stdout();
    let _ = term.clear_line();
    println!("Error: {}", file_path);
    println!("{}", error);
}

fn bytes_in_file_list(copy_info_list: &Vec<CopyInfo>) -> u64 {
    let mut byte_count = 0;
    for copy_info in copy_info_list {
        byte_count += copy_info.size;
    }
    byte_count
}
