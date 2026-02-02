use fuser::*;
use crate::filesystem::utils::*;

#[derive(Debug, Clone)]
pub struct UpdateCgroupFileFS {

}

impl UpdateCgroupFileFS {
    pub const NAME: &'static str = "update";
    pub const INODE: u64 = CGROUP_DIR_INODE + 3;

}

impl VirtualFS for UpdateCgroupFileFS { }

impl Filesystem for UpdateCgroupFileFS {

}

impl VirtualFile for UpdateCgroupFileFS {
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