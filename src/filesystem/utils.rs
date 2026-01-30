pub const ROOT_INODE: u64 = 1;
pub const RESERVED_INODE_SHIFT: u64 = 3;
pub const RESERVED_INODE_MASK: u64 = (1 << RESERVED_INODE_SHIFT) - 1;
pub const UNKNOWN_TIME: std::time::SystemTime = std::time::UNIX_EPOCH;
pub const DEFAULT_TTL: std::time::Duration = std::time::Duration::from_millis(1);
pub const DIR_NAME_SELF: &'static str = ".";
pub const DIR_NAME_PARENT: &'static str = "..";
pub const ROOT_UID: u32 = 0;
pub const ROOT_GID: u32 = 0;

pub fn inode_to_pid(inode: u64) -> Option<sysinfo::Pid> {
    u32::try_from(inode >> RESERVED_INODE_SHIFT)
        .map(|pid| sysinfo::Pid::from_u32(pid)).ok()
}

pub fn pid_to_dir_inode(pid: sysinfo::Pid) -> u64 {
    (pid.as_u32() as u64) << RESERVED_INODE_SHIFT
}

pub fn inode_refers_to_pid(inode: u64) -> bool {
    (inode & !RESERVED_INODE_MASK) != 0
}