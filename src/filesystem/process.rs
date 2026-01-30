use fuser::*;
use crate::filesystem::{VirtualFile, utils::*};
use crate::ProcessStats;

mod cgroup_file;
mod sched_policy_file;

use cgroup_file::*;
use sched_policy_file::*;

#[derive(Debug, Clone)]
pub struct ProcessFS<'a> {
    pub pid: sysinfo::Pid,
    pub stats: &'a ProcessStats,
    pub root_fs: super::root::RootAsParentFS<'a>,
    pub name: String,
}

impl<'a> ProcessFS<'a> {
    pub fn new(pid: sysinfo::Pid, stats: &'a ProcessStats, root_fs: super::root::RootAsParentFS<'a>) -> Self {
        Self { pid, stats, root_fs, name: format!("{}", pid) }
    }

    fn fs_from_file_name(&self, name: &std::ffi::OsStr) -> Option<Box<dyn VirtualFile + 'a>> {
        match name.to_str().unwrap() {
            DIR_NAME_SELF => panic!("recursion"),
            DIR_NAME_PARENT => Some(Box::new(self.root_fs.clone())),
            CgroupFileFS::NAME => Some(Box::new(CgroupFileFS { pid: self.pid, stats: self.stats })),
            SchedPolicyFileFS::NAME => Some(Box::new(SchedPolicyFileFS { pid: self.pid, stats: self.stats })),
            _ => None,
        }
    }

    fn fs_from_inode(&self, inode: u64) -> Option<Box<dyn VirtualFile + 'a>> {
        if !inode_refers_to_pid(inode) {
            return None;
        }

        match inode & RESERVED_INODE_MASK {
            0 => panic!("recursion"),
            CgroupFileFS::INODE_OFFSET => Some(Box::new(CgroupFileFS { pid: self.pid, stats: self.stats })),
            SchedPolicyFileFS::INODE_OFFSET => Some(Box::new(SchedPolicyFileFS { pid: self.pid, stats: self.stats })),
            _ => None,
        }
    }

    fn _readdir(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        _fh: u64,
        mut offset: i64,
        mut reply: ReplyDirectory,
    ) {
        if offset == 0 {
            let attr = self.attr();
            if reply.add(attr.ino, 1, attr.kind, DIR_NAME_SELF) {
                reply.ok();
                return;
            }

            offset += 1;
        }

        if offset == 1 {
            let attr = self.root_fs.attr();
            if reply.add(attr.ino, 2, attr.kind, DIR_NAME_PARENT) {
                reply.ok();
                return;
            }

            offset += 1;
        }

        for offset in (offset as u64) .. 1 << RESERVED_INODE_SHIFT {
            let Some(file) = self.fs_from_inode(ino + offset)
                else { continue; };

            let attr = file.attr();
            if reply.add(attr.ino, (offset + 1) as i64, attr.kind, file.name()) {
                break;
            }
        }

        reply.ok();
    }
}

impl Filesystem for ProcessFS<'_> {
    fn lookup(&mut self, _req: &Request<'_>, parent: u64, name: &std::ffi::OsStr, reply: ReplyEntry) {
        if parent == pid_to_dir_inode(self.pid) {
            if name == DIR_NAME_SELF {
                reply.entry(&DEFAULT_TTL, &self.attr(), 0);
            } else {
                let Some(file) = self.fs_from_file_name(name)
                    else { reply.error(libc::ENOENT); return; };

                reply.entry(&DEFAULT_TTL, &file.attr(), 0);
            }
        } else {
            let Some(mut file) = self.fs_from_file_name(name)
                else { reply.error(libc::ENOENT); return; };

            file.lookup(_req, parent, name, reply);
        }
    }

    fn getattr(&mut self, _req: &Request<'_>, ino: u64, fh: Option<u64>, reply: ReplyAttr) {
        if ino == pid_to_dir_inode(self.pid) {
            reply.attr(&DEFAULT_TTL, &self.attr());
        } else {
            let Some(mut file) = self.fs_from_inode(ino)
                else { reply.error(libc::ENOENT); return; };

            file.getattr(_req, ino, fh, reply);
        }
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
        if ino == pid_to_dir_inode(self.pid) {
            reply.error(libc::EISDIR);
        } else {
            let Some(mut file) = self.fs_from_inode(ino)
                else { reply.error(libc::ENOENT); return; };

            file.read(_req, ino, fh, offset, size, flags, lock_owner, reply);
        }
    }

    fn readdir(
            &mut self,
            _req: &Request<'_>,
            ino: u64,
            fh: u64,
            offset: i64,
            reply: ReplyDirectory,
        ) {
        if ino == pid_to_dir_inode(self.pid) {
            self._readdir(_req, ino, fh, offset, reply);
        } else {
            let Some(mut file) = self.fs_from_inode(ino)
                else { reply.error(libc::ENOENT); return; };

            file.readdir(_req, ino, fh, offset, reply);
        }
    }
}

impl VirtualFile for ProcessFS<'_> {
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