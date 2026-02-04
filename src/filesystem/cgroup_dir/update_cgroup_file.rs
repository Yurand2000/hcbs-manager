use fuser::*;
use crate::manager::Reservation;
use crate::filesystem::utils::*;

#[derive(Debug)]
pub struct UpdateCgroupFileFS<'a> {
    cgroup_manager: &'a mut crate::manager::HCBSManager,
}

impl<'a> UpdateCgroupFileFS<'a> {
    pub const NAME: &'static str = "update";
    pub const INODE: u64 = CGROUP_DIR_INODE + 3;

    pub fn new(cgroup_dir_fs: &'a mut super::CgroupDirFS<'_>) -> FileFS<Self> {
        FileFS::new( Self { cgroup_manager: cgroup_dir_fs.manager } )
    }

    fn parse_request(data: &str) -> Option<(&str, Reservation)> {
        use nom::Parser as _;
        use nom::character::complete::*;
        use nom::combinator::*;

        map(
            (
                crate::filesystem::utils::parser::parse_cgroup_name,
                space1,
                crate::filesystem::utils::parser::parse_cgroup_alloc_request,
            ),
            |(name, _, request)| (name, request)
        ).parse(data).map(|(_, res)| res).ok()
    }
}

impl FileFSInterface for UpdateCgroupFileFS<'_> {
    fn read_size(&self) -> anyhow::Result<usize> { anyhow::bail!("Cannot read from UpdateCgroupFile") }

    fn read_data(&self) -> anyhow::Result<&str> { anyhow::bail!("Cannot read from UpdateCgroupFile") }

    fn write_data(&mut self, data: &str) -> anyhow::Result<()> {
        let Some((name, request)) = Self::parse_request(data)
            else { anyhow::bail!("Invalid request"); };

        self.cgroup_manager.update_cgroup(name, request)
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