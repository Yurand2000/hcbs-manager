use fuser::*;

pub mod parent_dir_fs;
pub mod dir_fs;

pub use parent_dir_fs::ParentDirFS;
pub use dir_fs::{
    DirFS,
    DirFSInterface,
};

pub trait VirtualFile: Filesystem {
    fn inode(&self) -> u64;
    fn attr(&self) -> FileAttr;
    fn name(&self) -> &str;
}

pub const UNKNOWN_TIME: std::time::SystemTime = std::time::UNIX_EPOCH;
pub const DEFAULT_TTL: std::time::Duration = std::time::Duration::from_millis(1);
pub const DIR_NAME_SELF: &'static str = ".";
pub const DIR_NAME_PARENT: &'static str = "..";
pub const ROOT_UID: u32 = 0;
pub const ROOT_GID: u32 = 0;

/// INode Structure
/// 64 bits
/// | 2 bits Dir Type | 59 bits Dir Id | 3 bits Dir Files
///
/// Dir Types:
/// 0   RootFS
/// 1   Proc
///     Dir Id == PID
/// 2   CGroup
///     Dir Id == CGroup Name Hash 32-bit (?)
/// 3   -

pub const INODE_DIR_TYPE_SHIFT: u64 = 62;
pub const INODE_DIR_TYPE_MASK: u64 = 3 << INODE_DIR_TYPE_SHIFT;

pub const ROOT_INODE_DIR_TYPE: u64 = 0 << INODE_DIR_TYPE_SHIFT;
pub const PROC_INODE_DIR_TYPE: u64 = 1 << INODE_DIR_TYPE_SHIFT;
pub const CGROUP_INODE_DIR_TYPE: u64 = 2 << INODE_DIR_TYPE_SHIFT;

pub const INODE_DIR_ID_SHIFT: u64 = 3;
pub const INODE_DIR_ID_MASK: u64 = ((1 << INODE_DIR_TYPE_SHIFT) - 1) & !INODE_DIR_FILE_MASK;

pub const INODE_DIR_FILE_MASK: u64 = (1 << INODE_DIR_ID_SHIFT) - 1;
pub const INODE_DIR_FILE_MAX: u64 = 1 << INODE_DIR_ID_SHIFT;

/// Known INodes
pub const ROOT_DIR_INODE: u64 = ROOT_INODE_DIR_TYPE | 1;
pub const PROC_DIR_INODE: u64 = PROC_INODE_DIR_TYPE;
pub const CGROUP_DIR_INODE: u64 = CGROUP_INODE_DIR_TYPE;

pub fn inode_is_pid(inode: u64) -> bool {
    (inode & INODE_DIR_TYPE_MASK) == PROC_INODE_DIR_TYPE
}

pub fn inode_to_pid_dir(inode: u64) -> Option<sysinfo::Pid> {
    if !inode_is_pid(inode) {
        return None;
    }

    u32::try_from((inode & INODE_DIR_ID_MASK) >> INODE_DIR_ID_SHIFT)
        .map(|pid| sysinfo::Pid::from_u32(pid)).ok()
}

pub fn pid_to_dir_inode(pid: sysinfo::Pid) -> u64 {
    (pid.as_u32() as u64) << INODE_DIR_ID_SHIFT | PROC_INODE_DIR_TYPE
}