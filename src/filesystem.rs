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

        RootFS { active_procs: &self.active_procs }
            .lookup(_req, parent, name, reply);
    }

    fn getattr(&mut self, _req: &Request<'_>, ino: u64, fh: Option<u64>, reply: ReplyAttr) {
        self.update_active_processes();

        RootFS { active_procs: &self.active_procs }
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

        RootFS { active_procs: &self.active_procs }
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

        RootFS { active_procs: &self.active_procs }
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

        RootFS { active_procs: &self.active_procs }
            .readdir(_req, ino, fh, offset, reply);
    }
}


#[derive(Debug, Clone)]
pub struct RootFS<'a> {
    pub active_procs: &'a HashMap<sysinfo::Pid, ProcessStats>,
}

impl<'a> RootFS<'a> {
    fn fs_from_file_name(&'a self, name: &std::ffi::OsStr) -> Option<Box<dyn VirtualFile + 'a>> {
        match name.to_str().unwrap() {
            DIR_NAME_SELF => panic!("recursion"),
            ProcDirFS::NAME => Some(Box::new(ProcDirFS { active_procs: self.active_procs, parent_fs: ParentDirFS::new(self) })),
            CgroupDirFS::NAME => Some(Box::new(CgroupDirFS { parent_fs: ParentDirFS::new(self) })),
            _ => None,
        }
    }

    fn fs_from_inode(&'a self, inode: u64) -> Option<Box<dyn VirtualFile + 'a>> {
        match inode & INODE_DIR_TYPE_MASK {
            ROOT_DIR_INODE => match inode & INODE_DIR_FILE_MASK {
                0 => panic!("inode zero"),
                1 => panic!("recursion"),
                _ => None,
            },
            PROC_DIR_INODE => Some(Box::new(ProcDirFS { active_procs: self.active_procs, parent_fs: ParentDirFS::new(self) })),
            CGROUP_DIR_INODE => Some(Box::new(CgroupDirFS { parent_fs: ParentDirFS::new(self) })),
            _ => None,
        }
    }

    fn _readdir(
        &mut self,
        _req: &Request<'_>,
        _ino: u64,
        _fh: u64,
        mut offset: i64,
        mut reply: ReplyDirectory,
    ) {
        if offset == 0 {
            let attr = self.attr();
            if reply.add(attr.ino, (offset + 1) as i64, attr.kind, self.name()) {
                reply.ok();
                return;
            }

            offset += 1;
        }

        if offset == 1 {
            let proc = ProcDirFS { active_procs: self.active_procs, parent_fs: ParentDirFS::new(self) };
            let attr = proc.attr();
            if reply.add(attr.ino, (offset + 1) as i64, attr.kind, proc.name()) {
                reply.ok();
                return;
            }

            offset += 1;
        }

        if offset == 2 {
            let cgroup = CgroupDirFS { parent_fs: ParentDirFS::new(self) };
            let attr = cgroup.attr();
            if reply.add(attr.ino, (offset + 1) as i64, attr.kind, cgroup.name()) {
                reply.ok();
                return;
            }
        }

        reply.ok();
        return;
    }
}

impl Filesystem for RootFS<'_> {
    fn lookup(&mut self, _req: &Request<'_>, parent: u64, name: &std::ffi::OsStr, reply: ReplyEntry) {
        if parent == ROOT_DIR_INODE {
            if name == DIR_NAME_SELF {
                reply.entry(&DEFAULT_TTL, &self.attr(), 0);
            } else {
                let Some(file) = self.fs_from_file_name(name)
                    else { reply.error(libc::ENOENT); return; };

                reply.entry(&DEFAULT_TTL, &file.attr(), 0);
            }
        } else {
            let Some(mut file) = self.fs_from_inode(parent)
                else { reply.error(libc::ENOENT); return; };

            file.lookup(_req, parent, name, reply);
        }
    }

    fn getattr(&mut self, _req: &Request<'_>, ino: u64, fh: Option<u64>, reply: ReplyAttr) {
        if ino == ROOT_DIR_INODE {
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
        if ino == ROOT_DIR_INODE {
            reply.error(libc::EISDIR);
        } else {
            let Some(mut file) = self.fs_from_inode(ino)
                else { reply.error(libc::ENOENT); return; };

            file.read(_req, ino, fh, offset, size, flags, lock_owner, reply);
        }
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
        if ino == ROOT_DIR_INODE {
            reply.error(libc::EISDIR);
        } else {
            let Some(mut file) = self.fs_from_inode(ino)
                else { reply.error(libc::ENOENT); return; };

            file.write(_req, ino, fh, offset, data, write_flags, flags, lock_owner, reply);
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
        if ino == ROOT_DIR_INODE {
            self._readdir(_req, ino, fh, offset, reply);
        } else {
            let Some(mut file) = self.fs_from_inode(ino)
                else { reply.error(libc::ENOENT); return; };

            file.readdir(_req, ino, fh, offset, reply);
        }
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