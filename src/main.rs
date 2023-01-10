use std::fs::{copy, create_dir, read_dir, read_link};
use std::io::Error;
use std::os::unix::fs::symlink;
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

fn copy_file_list(split_file_info: SplitCopyInfo) {
    for copy_info in split_file_info.copy_info_list {
        if !copy_info.is_symlink {
            // File
            if let Err(err) = copy(copy_info.src_path, copy_info.dst_path) {
                println!("Err: {}", err);
            }
        } else {
            // SymLink
            match read_link(copy_info.src_path) {
                Ok(original) => {
                    if let Err(err) = symlink(original, copy_info.dst_path) {
                        println!("Err: {}", err);
                    }
                }
                Err(err) => println!("Err: {}", err),
            }
        }
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

fn main() {
    let mut copy_info_list = Vec::<CopyInfo>::new();
    if let Err(err) = scan_source_prep_dest(
        "/Users/davidvernon/.rustup",
        "/Users/davidvernon/rustup_copy",
        &mut copy_info_list,
    ) {
        println!("{}", err);
    }
    let bytes_copied = bytes_to_human_readable(bytes_in_file_list(&copy_info_list));

    let split_file_info_list = balanced_split(copy_info_list, 12);

    let mut join_handle_list = Vec::<JoinHandle<()>>::new();
    for split_file_info in split_file_info_list {
        let join_handle = thread::spawn(|| {
            copy_file_list(split_file_info);
        });
        join_handle_list.push(join_handle);
    }

    for join_handle in join_handle_list {
        let _ = join_handle.join();
    }

    println!("{} copied.", bytes_copied);
}
