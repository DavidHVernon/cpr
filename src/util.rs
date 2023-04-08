use crate::types::*;
use std::process::exit;

// Given a number of bytes, convert into a human readable
// string.
pub fn human_readable_bytes(bytes: u64) -> String {
    if bytes < 1000 {
        return format!("{} bytes", bytes);
    } else if bytes < 1_000_000 {
        return format!("{:.2} KB", bytes as f64 / 1000.0);
    } else if bytes < 1_000_000_000 {
        return format!("{:.2} MB", bytes as f64 / 1_000_000.0);
    } else {
        return format!("{:.2} GB", bytes as f64 / 1_000_000_000.0);
    }
}

// Given one copy_info_list, split it into n split_copy_info structs where each
// one has roughly the same number of bytes in it.
pub fn balanced_split(copy_info_list: Vec<CopyInfo>, n: i32) -> Vec<SplitCopyInfo> {
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

pub fn home_dir() -> String {
    if let Some(home_dir) = dirs::home_dir() {
        return home_dir.to_string_lossy().to_string();
    } else {
        print_error_and_exit("Could not determine home dir.");
        return String::from("");
    }
}

pub fn print_error_and_exit(error_str: &str) {
    println!("{}", error_str);

    exit(-1);
}
