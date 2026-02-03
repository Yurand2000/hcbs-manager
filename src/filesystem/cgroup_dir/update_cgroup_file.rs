use fuser::*;
use crate::filesystem::utils::*;

#[derive(Debug)]
pub struct UpdateCgroupFileFS<'a> {
    cgroup_manager: &'a mut crate::manager::CgroupManager,
}

impl<'a> UpdateCgroupFileFS<'a> {
    pub const NAME: &'static str = "update";
    pub const INODE: u64 = CGROUP_DIR_INODE + 3;

    pub fn new(cgroup_manager: &'a mut crate::manager::CgroupManager) -> FileFS<Self> {
        FileFS::new( Self { cgroup_manager } )
    }
}

impl FileFSInterface for UpdateCgroupFileFS<'_> {
    fn read_size(&self) -> anyhow::Result<usize> { anyhow::bail!("Cannot read from UpdateCgroupFile") }

    fn read_data(&self) -> anyhow::Result<&str> { anyhow::bail!("Cannot read from UpdateCgroupFile") }

    fn write_data(&mut self, _: &str) -> anyhow::Result<()> {
        anyhow::bail!("Cannot write to UpdateCgroupFile")
    }
}

impl VirtualFile for UpdateCgroupFileFS<'_> {
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