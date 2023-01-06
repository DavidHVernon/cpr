use std::fs::{read_dir, ReadDir};
use std::io::Error;

struct FileInfo {
    pub path: String,
    pub size: u64,
}

struct SplitFileInfo {
    pub size: u64,
    pub file_list: Vec<FileInfo>,
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

fn scan_dir(dir_path: &str, file_list: &mut Vec<FileInfo>) -> Result<(), Error> {
    for entry in read_dir(dir_path)? {
        let entry = entry?;
        let meta = entry.metadata()?;

        if let Some(dir_path) = entry.path().as_os_str().to_str() {
            if meta.is_dir() {
                scan_dir(dir_path, file_list);
            } else {
                file_list.push(FileInfo {
                    path: dir_path.to_string(),
                    size: meta.len(),
                });
            }
        }
    }

    Ok(())
}

fn balanced_split(file_info_list: Vec<FileInfo>, n: i32) -> Vec<SplitFileInfo> {
    let mut split_file_info_list = Vec::<SplitFileInfo>::new();
    for i in 0..n {
        split_file_info_list.push(SplitFileInfo {
            size: 0,
            file_list: Vec::<FileInfo>::new(),
        });
    }

    for file_info in file_info_list {
        // find the smallest bucket
        let (mut index, mut min_size) = (0 as u64, u64::MAX);
        let mut i = 0;
        for file_info in &split_file_info_list {
            if file_info.size < min_size {
                (index, min_size) = (i, file_info.size);
            }
            i += 1;
        }

        let split_file_info = split_file_info_list
            .get_mut(index as usize)
            .expect("index out of bounds.");
        split_file_info.size += file_info.size;
        split_file_info.file_list.push(file_info);
    }

    split_file_info_list
}

fn main() {
    let mut file_list = Vec::<FileInfo>::new();
    if let Err(err) = scan_dir("/Users/davidvernon/.cargo", &mut file_list) {
        println!("{}", err);
    }

    let mut total_size = 0;
    let count = file_list.len();
    for file in &file_list {
        total_size += file.size;
        println!("{}: {}", file.size, &file.path);
    }
    println!("{} files - {}", count, bytes_to_human_readable(total_size));

    let split_file_info = balanced_split(file_list, 6);
    for split_file_info in split_file_info {
        println!(
            "{}: {}",
            bytes_to_human_readable(split_file_info.size),
            split_file_info.file_list.len()
        )
    }
}
