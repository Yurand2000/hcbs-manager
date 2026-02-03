use fuser::*;
use hcbs_utils::prelude::*;
use crate::filesystem::utils::*;
use crate::ProcessStats;

pub struct CgroupFileFS<'a> {
    pid: sysinfo::Pid,
    stats: &'a ProcessStats,
    cgroup: Option<String>,
}

impl<'a> CgroupFileFS<'a> {
    pub const NAME: &'static str = "cgroup";
    pub const INODE_OFFSET: u64 = 2;

    pub fn new(pid_dir_fs: &'a super::PidDirFS) -> FileFS<Self> {
        let cgroup = get_pid_cgroup(pid_dir_fs.pid.as_u32())
                        .map(|mut str| { str += "\n"; str }).ok();

        FileFS::new( Self { pid: pid_dir_fs.pid, stats: pid_dir_fs.stats, cgroup } )
    }

    fn parse_request(data: &str) -> Option<&str> {
        crate::filesystem::utils::
            parser::parse_cgroup_name(data).map(|(_, res)| res).ok()
    }
}

impl FileFSInterface for CgroupFileFS<'_> {
    fn read_size(&self) -> anyhow::Result<usize> {
        self.cgroup.as_ref()
            .map(|str| str.len())
            .ok_or_else(|| anyhow::anyhow!("Cgroup not found") )
    }

    fn read_data(&self) -> anyhow::Result<&str> {
        self.cgroup.as_ref()
            .map(|str| str.as_str())
            .ok_or_else(|| anyhow::anyhow!("Cgroup not found") )
    }

    fn write_data(&mut self, data: &str) -> anyhow::Result<()> {
        let Some(name) = Self::parse_request(data)
            else { anyhow::bail!("Invalid request"); };

        if !get_sched_policy(self.pid.as_u32())?.is_other() {
            return Err(anyhow::format_err!("Only SCHED_OTHER processes are allowed to migrate."))
        }

        assign_pid_to_cgroup(name, self.pid.as_u32())
    }
}

impl VirtualFile for CgroupFileFS<'_> {
    fn inode(&self) -> u64 {
        pid_to_dir_inode(self.pid) + Self::INODE_OFFSET
    }

    fn attr(&self) -> FileAttr {
        FileAttr {
            ino: self.inode(),
            size: 0,
            blocks: 0,
            atime: self.stats.crtime,
            mtime: self.stats.crtime,
            ctime: self.stats.crtime,
            crtime: self.stats.crtime,
            kind: FileType::RegularFile,
            perm: 0o664,
            nlink: 1,
            uid: *self.stats.uid,
            gid: *self.stats.gid,
            rdev: 0,
            blksize: 512,
            flags: 0,
        }
    }

    fn name(&self) -> &str {
        Self::NAME
    }
}