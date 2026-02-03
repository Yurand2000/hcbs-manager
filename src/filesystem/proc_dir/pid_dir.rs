use fuser::*;
use crate::filesystem::utils::*;
use crate::ProcessStats;

mod cgroup_file;
mod sched_policy_file;

use cgroup_file::*;
use sched_policy_file::*;

#[derive(Debug, Clone)]
pub struct PidDirFS<'a> {
    pid: sysinfo::Pid,
    stats: &'a ProcessStats,
    name: String,
    cgroup_manager: &'a crate::manager::CgroupManager,
    proc_dir: &'a super::ProcDirFS<'a>,
}

impl<'a> PidDirFS<'a> {
    pub fn new(
        pid: sysinfo::Pid,
        stats: &'a ProcessStats,
        proc_dir: &'a super::ProcDirFS<'a>
    ) -> DirFS<Self> {
        DirFS::new( Self { pid, stats, name: format!("{}", pid), cgroup_manager: proc_dir.cgroup_manager, proc_dir } )
    }
}

impl DirFSInterface for PidDirFS<'_> {
    fn parent_attr(&self) -> Option<FileAttr> {
        Some(self.proc_dir.attr())
    }

    fn fs_from_file_name<'a>(&'a mut self, name: &std::ffi::OsStr) -> Option<Box<dyn VirtualFS + 'a>> {
        match name.to_str().unwrap() {
            CgroupFileFS::NAME => Some(Box::new(CgroupFileFS::new(self))),
            SchedPolicyFileFS::NAME => Some(Box::new(SchedPolicyFileFS::new(self))),
            _ => None,
        }
    }

    fn fs_from_inode<'a>(&'a mut self, inode: u64) -> Option<Box<dyn VirtualFS + 'a>> {
        if !inode_is_pid(inode) {
            return None;
        }

        match inode & INODE_DIR_FILE_MASK {
            0 => panic!("recursion"),
            CgroupFileFS::INODE_OFFSET => Some(Box::new(CgroupFileFS::new(self))),
            SchedPolicyFileFS::INODE_OFFSET => Some(Box::new(SchedPolicyFileFS::new(self))),
            _ => None,
        }
    }

    fn fs_inodes_in_dir(&self) -> impl Iterator<Item = u64> {
        [
            CgroupFileFS::INODE_OFFSET,
            SchedPolicyFileFS::INODE_OFFSET,
        ].into_iter().map(|offset| self.inode() + offset)
    }
}

impl VirtualFile for PidDirFS<'_> {
    fn inode(&self) -> u64 {
        pid_to_dir_inode(self.pid)
    }

    fn attr(&self) -> FileAttr {
        let inode = pid_to_dir_inode(self.pid);

        FileAttr {
            ino: inode,
            size: 0,
            blocks: 0,
            atime: self.stats.crtime,
            mtime: self.stats.crtime,
            ctime: self.stats.crtime,
            crtime: self.stats.crtime,
            kind: FileType::Directory,
            perm: 0o775,
            nlink: 1,
            uid: *self.stats.uid,
            gid: *self.stats.gid,
            rdev: 0,
            blksize: 512,
            flags: 0,
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}