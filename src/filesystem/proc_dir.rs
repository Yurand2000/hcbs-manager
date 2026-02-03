use fuser::*;
use std::collections::HashMap;
use crate::ProcessStats;
use crate::filesystem::utils::*;

mod pid_dir;

use pid_dir::*;

#[derive(Debug, Clone)]
pub struct ProcDirFS<'a> {
    active_procs: &'a HashMap<sysinfo::Pid, ProcessStats>,
    cgroup_manager: &'a crate::manager::CgroupManager,
    root_fs: &'a super::RootFS<'a>,
}

impl<'a> ProcDirFS<'a> {
    pub const NAME: &'static str = "proc";

    pub fn new(root_fs: &'a super::RootFS) -> DirFS<Self> {
        DirFS::new( Self { root_fs, active_procs: root_fs.active_procs, cgroup_manager: root_fs.cgroup_manager } )
    }

    fn process_from_name(&'a self, name: &str) -> Option<(sysinfo::Pid, &'a ProcessStats)> {
        let pid = sysinfo::Pid::from_u32(name.parse::<u32>().ok()?);
        let stats = self.active_procs.get(&pid)?;

        Some((pid, stats))
    }

    fn process_from_inode(&'a self, inode: u64) -> Option<(sysinfo::Pid, &'a ProcessStats)> {
        let pid = inode_to_pid_dir(inode)?;
        let stats = self.active_procs.get(&pid)?;

        Some((pid, stats))
    }
}

impl DirFSInterface for ProcDirFS<'_> {
    fn parent_attr(&self) -> Option<FileAttr> {
        Some(self.root_fs.attr())
    }

    fn fs_from_file_name<'a>(&'a mut self, name: &std::ffi::OsStr) -> Option<Box<dyn VirtualFS + 'a>> {
        self.process_from_name(name.to_str().unwrap())
            .map(|(pid, stats)| -> Box<dyn VirtualFS> {
                Box::new(PidDirFS::new(pid, stats, self))
            })
    }

    fn fs_from_inode<'a>(&'a mut self, inode: u64) -> Option<Box<dyn VirtualFS + 'a>> {
        if inode != PROC_DIR_INODE {
            self.process_from_inode(inode)
                .map(|(pid, stats)| -> Box<dyn VirtualFS + 'a> {
                    Box::new(PidDirFS::new(pid, stats, self))
                })
        } else {
            match inode & INODE_DIR_FILE_MASK {
                0 => panic!("recursion"),
                _ => None,
            }
        }
    }

    fn fs_inodes_in_dir(&self) -> impl Iterator<Item = u64> {
        self.active_procs.iter()
            .map(|(&pid, _)| pid_to_dir_inode(pid))
    }
}

impl<'a> VirtualFile for ProcDirFS<'a> {
    fn inode(&self) -> u64 {
        PROC_DIR_INODE
    }

    fn attr(&self) -> FileAttr {
        FileAttr {
            ino: PROC_DIR_INODE,
            size: 0,
            blocks: 0,
            atime: UNKNOWN_TIME,
            mtime: UNKNOWN_TIME,
            ctime: UNKNOWN_TIME,
            crtime: UNKNOWN_TIME,
            kind: FileType::Directory,
            perm: 0o775,
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