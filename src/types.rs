pub struct Arguments {
    pub src_path: String,
    pub dst_path: String,
}

pub struct CopyInfo {
    pub src_path: String,
    pub dst_path: String,
    pub is_symlink: bool,
    pub size: u64,
}

pub struct SplitCopyInfo {
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
