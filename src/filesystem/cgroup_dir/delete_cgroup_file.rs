use fuser::*;
use crate::filesystem::utils::*;

#[derive(Debug)]
pub struct DeleteCgroupFileFS<'a> {
    cgroup_manager: &'a mut crate::manager::CgroupManager,
}

impl<'a> DeleteCgroupFileFS<'a> {
    pub const NAME: &'static str = "delete";
    pub const INODE: u64 = CGROUP_DIR_INODE + 2;

    pub fn new(cgroup_manager: &'a mut crate::manager::CgroupManager) -> FileFS<Self> {
        FileFS::new( Self { cgroup_manager } )
    }

    fn parse_request(data: &str) -> Option<&str> {
        crate::filesystem::utils::
            parser::parse_cgroup_name(data).map(|(_, res)| res).ok()
    }
}

impl FileFSInterface for DeleteCgroupFileFS<'_> {
    fn read_size(&self) -> anyhow::Result<usize> { anyhow::bail!("Cannot read from DeleteCgroupFile") }

    fn read_data(&self) -> anyhow::Result<&str> { anyhow::bail!("Cannot read from DeleteCgroupFile") }

    fn write_data(&mut self, data: &str) -> anyhow::Result<()> {
        let Some(name) = Self::parse_request(data)
            else { anyhow::bail!("Invalid request"); };

        self.cgroup_manager.destroy_cgroup(name)
    }
}

impl VirtualFile for DeleteCgroupFileFS<'_> {
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