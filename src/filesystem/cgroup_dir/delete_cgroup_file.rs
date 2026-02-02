use fuser::*;
use crate::filesystem::utils::*;

#[derive(Debug, Clone)]
pub struct DeleteCgroupFileFS {

}

impl DeleteCgroupFileFS {
    pub const NAME: &'static str = "delete";
    pub const INODE: u64 = CGROUP_DIR_INODE + 2;

}

impl Filesystem for DeleteCgroupFileFS {

}

impl VirtualFile for DeleteCgroupFileFS {
    fn inode(&self) -> u64 {
        Self::INODE
    }

    fn attr(&self) -> FileAttr {
        FileAttr {
            ino: Self::INODE,
            size: 0,
            blocks: 0,
            atime: UNKNOWN_TIME,
            mtime: UNKNOWN_TIME,
            ctime: UNKNOWN_TIME,
            crtime: UNKNOWN_TIME,
            kind: FileType::RegularFile,
            perm: 0o666,
            nlink: 1,
            uid: ROOT_UID,
            gid: ROOT_GID,
            rdev: 0,
            blksize: 512,
            flags: 0,
        }
    }

    fn name(&self) -> &str {
        Self::NAME
    }
}