use fuser::*;
use std::collections::HashMap;
use crate::filesystem::utils::*;
use crate::ProcessStats;

mod proc_dir;
mod cgroup_dir;
mod utils;

use proc_dir::*;
use cgroup_dir::*;

impl Filesystem for super::HCBSController {
    fn lookup(&mut self, _req: &Request<'_>, parent: u64, name: &std::ffi::OsStr, reply: ReplyEntry) {
        self.update_active_processes();

        RootFS::new(&self.active_procs)
            .lookup(_req, parent, name, reply);
    }

    fn getattr(&mut self, _req: &Request<'_>, ino: u64, fh: Option<u64>, reply: ReplyAttr) {
        self.update_active_processes();

        RootFS::new(&self.active_procs)
            .getattr(_req, ino, fh, reply);
    }

    fn read(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        fh: u64,
        offset: i64,
        size: u32,
        flags: i32,
        lock_owner: Option<u64>,
        reply: ReplyData,
    ) {
        self.update_active_processes();

        RootFS::new(&self.active_procs)
            .read(_req, ino, fh, offset, size, flags, lock_owner, reply);
    }

    fn write(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        fh: u64,
        offset: i64,
        data: &[u8],
        write_flags: u32,
        flags: i32,
        lock_owner: Option<u64>,
        reply: ReplyWrite,
    ) {
        self.update_active_processes();

        RootFS::new(&self.active_procs)
            .write(_req, ino, fh, offset, data, write_flags, flags, lock_owner, reply);
    }

    fn readdir(
            &mut self,
            _req: &Request<'_>,
            ino: u64,
            fh: u64,
            offset: i64,
            reply: ReplyDirectory,
        ) {
        self.update_active_processes();

        RootFS::new(&self.active_procs)
            .readdir(_req, ino, fh, offset, reply);
    }
}


#[derive(Debug, Clone)]
pub struct RootFS<'a> {
    active_procs: &'a HashMap<sysinfo::Pid, ProcessStats>,
}

impl<'a> RootFS<'a> {
    pub fn new(active_procs: &'a HashMap<sysinfo::Pid, ProcessStats>) -> DirNoParentFS<Self> {
        DirNoParentFS { implementor: Self { active_procs } }
    }
}

impl DirFSInterface for RootFS<'_> {
    fn fs_from_file_name<'a>(&'a self, name: &std::ffi::OsStr) -> Option<Box<dyn VirtualFS + 'a>> {
        match name.to_str().unwrap() {
            ProcDirFS::NAME => Some(Box::new(ProcDirFS::new(self.active_procs, ParentDirFS::new(self)))),
            CgroupDirFS::NAME => Some(Box::new(CgroupDirFS::new(ParentDirFS::new(self)))),
            _ => None,
        }
    }

    fn fs_from_inode<'a>(&'a self, inode: u64) -> Option<Box<dyn VirtualFS + 'a>> {
        match inode & INODE_DIR_TYPE_MASK {
            ROOT_DIR_INODE => match inode & INODE_DIR_FILE_MASK {
                0 => panic!("inode zero"),
                1 => panic!("recursion"),
                _ => None,
            },
            PROC_DIR_INODE => Some(Box::new(ProcDirFS::new(self.active_procs, ParentDirFS::new(self)))),
            CGROUP_DIR_INODE => Some(Box::new(CgroupDirFS::new(ParentDirFS::new(self)))),
            _ => None,
        }
    }

    fn readdir_files<'a>(&'a self) -> impl Iterator<Item = Box<dyn VirtualFS + 'a>> {
        let files: [Box<dyn VirtualFS>; _] = [
            Box::new(ProcDirFS::new(self.active_procs, ParentDirFS::new(self))),
            Box::new(CgroupDirFS::new(ParentDirFS::new(self))),
        ];

        files.into_iter()
    }
}

impl VirtualFile for RootFS<'_> {
    fn inode(&self) -> u64 {
        ROOT_DIR_INODE
    }

    fn attr(&self) -> FileAttr {
        FileAttr {
            ino: ROOT_DIR_INODE,
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
        DIR_NAME_SELF
    }
}