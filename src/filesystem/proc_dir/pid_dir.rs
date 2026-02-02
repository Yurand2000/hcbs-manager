use fuser::*;
use crate::filesystem::utils::*;
use crate::ProcessStats;

mod cgroup_file;
mod sched_policy_file;

use cgroup_file::*;
use sched_policy_file::*;

#[derive(Debug, Clone)]
pub struct PidDirFS<'a> {
    pub pid: sysinfo::Pid,
    pub stats: &'a ProcessStats,
    pub name: String,
}

impl<'a> PidDirFS<'a> {
    pub fn new(pid: sysinfo::Pid, stats: &'a ProcessStats, parent_fs: ParentDirFS<'a>) -> DirFS<'a, Self> {
        DirFS {
            implementor: Self { pid, stats, name: format!("{}", pid) },
            parent_fs,
        }
    }
}

impl DirFSInterface for PidDirFS<'_> {
    fn fs_from_file_name<'a>(&'a self, name: &std::ffi::OsStr) -> Option<Box<dyn VirtualFS + 'a>> {
        match name.to_str().unwrap() {
            CgroupFileFS::NAME => Some(Box::new(CgroupFileFS { pid: self.pid, stats: self.stats })),
            SchedPolicyFileFS::NAME => Some(Box::new(SchedPolicyFileFS { pid: self.pid, stats: self.stats })),
            _ => None,
        }
    }

    fn fs_from_inode<'a>(&'a self, inode: u64) -> Option<Box<dyn VirtualFS + 'a>> {
        if !inode_is_pid(inode) {
            return None;
        }

        match inode & INODE_DIR_FILE_MASK {
            0 => panic!("recursion"),
            CgroupFileFS::INODE_OFFSET => Some(Box::new(CgroupFileFS { pid: self.pid, stats: self.stats })),
            SchedPolicyFileFS::INODE_OFFSET => Some(Box::new(SchedPolicyFileFS { pid: self.pid, stats: self.stats })),
            _ => None,
        }
    }

    fn readdir_files<'a>(&'a self) -> impl Iterator<Item = Box<dyn VirtualFS + 'a>> {
        let files: [Box<dyn VirtualFS>; _] = [
            Box::new(CgroupFileFS { pid: self.pid, stats: self.stats }),
            Box::new(SchedPolicyFileFS { pid: self.pid, stats: self.stats }),
        ];

        files.into_iter()
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