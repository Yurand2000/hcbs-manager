use fuser::*;
use crate::filesystem::utils::*;

#[derive(Debug, Clone)]
pub struct CreateCgroupFileFS<'a> {
    pub parent_fs: ParentDirFS<'a>,
}

impl<'a> CreateCgroupFileFS<'a> {
    pub const NAME: &'static str = "create";
    pub const INODE: u64 = CGROUP_DIR_INODE + 1;

}

impl Filesystem for CreateCgroupFileFS<'_> {

}

impl VirtualFile for CreateCgroupFileFS<'_> {
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