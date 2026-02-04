use std::collections::HashMap;

use fuser::*;
use crate::ProcessStats;
use crate::filesystem::utils::*;

mod pid_dir;

use pid_dir::*;

#[derive(Debug)]
pub struct ProcDirFS<'a> {
    active_procs: &'a HashMap<sysinfo::Pid, ProcessStats>,
    manager: &'a mut crate::manager::HCBSManager,
    root_fs_attr: FileAttr,
}

impl<'a> ProcDirFS<'a> {
    pub const NAME: &'static str = "proc";

    pub fn new(root_fs: &'a mut super::RootFS<'_>) -> DirFS<Self> {
        let root_fs_attr = root_fs.attr();

        DirFS::new( Self {
            active_procs: root_fs.active_procs,
            manager: root_fs.manager,
            root_fs_attr,
        } )
    }
}

impl DirFSInterface for ProcDirFS<'_> {
    fn parent_attr(&self) -> Option<FileAttr> {
        Some(self.root_fs_attr)
    }

    fn fs_from_file_name<'a>(&'a mut self, name: &std::ffi::OsStr) -> Option<Box<dyn VirtualFS + 'a>> {
        PidDirFS::new_from_name(self, name.to_str().unwrap())
            .map(|fs| -> Box<dyn VirtualFS + 'a> { Box::new(fs) })
    }

    fn fs_from_inode<'a>(&'a mut self, inode: u64) -> Option<Box<dyn VirtualFS + 'a>> {
        if inode != PROC_DIR_INODE {
            PidDirFS::new_from_inode(self, inode)
                .map(|fs| -> Box<dyn VirtualFS + 'a> { Box::new(fs) })
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

impl VirtualFile for ProcDirFS<'_> {
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