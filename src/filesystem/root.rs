use std::collections::HashMap;

use fuser::*;
use crate::{ProcessStats, filesystem::VirtualFile};
use crate::filesystem::utils::*;
use crate::filesystem::process::ProcessFS;

#[derive(Debug, Clone)]
pub struct RootFS<'a> {
    pub active_procs: &'a HashMap<sysinfo::Pid, ProcessStats>,
}

impl<'a> RootFS<'a> {
    fn process_from_name(&'a self, name: &str) -> Option<(sysinfo::Pid, &'a ProcessStats)> {
        let pid = sysinfo::Pid::from_u32(name.parse::<u32>().ok()?);
        let stats = self.active_procs.get(&pid)?;

        Some((pid, stats))
    }

    fn process_from_inode(&'a self, inode: u64) -> Option<(sysinfo::Pid, &'a ProcessStats)> {
        let pid = inode_to_pid(inode)?;
        let stats = self.active_procs.get(&pid)?;

        Some((pid, stats))
    }

    fn fs_from_file_name(&'a self, name: &std::ffi::OsStr) -> Option<Box<dyn VirtualFile + 'a>> {
        let name = name.to_str().unwrap();

        if name == DIR_NAME_SELF {
            panic!("recursion");
        }

        self.process_from_name(name)
            .map(|(pid, stats)| -> Box<dyn VirtualFile + 'a> {
                Box::new(ProcessFS::new(pid, stats, RootAsParentFS { root_fs: self }))
            })
    }

    fn fs_from_inode(&'a self, inode: u64) -> Option<Box<dyn VirtualFile + 'a>> {
        if inode_refers_to_pid(inode) {
            self.process_from_inode(inode)
                .map(|(pid, stats)| -> Box<dyn VirtualFile + 'a> {
                    Box::new(ProcessFS::new(pid, stats, RootAsParentFS { root_fs: self }))
                })
        } else {
            match inode & RESERVED_INODE_MASK {
                0 => panic!("recursion"),
                _ => None,
            }
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

        for (i, (&pid, stats)) in self.active_procs.iter().enumerate().skip((offset - 1) as usize) {
            let offset = i + 1;
            let proc = ProcessFS::new(pid, stats, RootAsParentFS { root_fs: self });
            let attr = proc.attr();

            if reply.add(attr.ino, (offset + 1) as i64, attr.kind, proc.name()) {
                break;
            }
        }

        reply.ok();
        return;
    }
}

impl Filesystem for RootFS<'_> {
    fn lookup(&mut self, _req: &Request<'_>, parent: u64, name: &std::ffi::OsStr, reply: ReplyEntry) {
        if parent == ROOT_INODE {
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
        if ino == ROOT_INODE {
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
        if ino == ROOT_INODE {
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
        if ino == ROOT_INODE {
            self._readdir(_req, ino, fh, offset, reply);
        } else {
            let Some(mut file) = self.fs_from_inode(ino)
                else { reply.error(libc::ENOENT); return; };

            file.readdir(_req, ino, fh, offset, reply);
        }
    }
}

impl VirtualFile for RootFS<'_> {
    fn attr(&self) -> FileAttr {
        FileAttr {
            ino: ROOT_INODE,
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

#[derive(Debug, Clone)]
pub struct RootAsParentFS<'a> {
    pub root_fs: &'a RootFS<'a>,
}

impl<'a> RootAsParentFS<'a> {

}

impl Filesystem for RootAsParentFS<'_> {
    fn lookup(&mut self, _req: &Request<'_>, _parent: u64, _name: &std::ffi::OsStr, reply: ReplyEntry) {
        reply.entry(&DEFAULT_TTL, &self.attr(), 0);
    }

    fn getattr(&mut self, _req: &Request<'_>, _ino: u64, _fh: Option<u64>, reply: ReplyAttr) {
        reply.attr(&DEFAULT_TTL, &self.attr());
    }
}

impl VirtualFile for RootAsParentFS<'_> {
    fn attr(&self) -> FileAttr {
        self.root_fs.attr()
    }

    fn name(&self) -> &str {
        panic!("unexpected")
    }
}