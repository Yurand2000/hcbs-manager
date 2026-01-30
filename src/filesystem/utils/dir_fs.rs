use fuser::*;
use super::*;

#[derive(Debug, Clone)]
pub struct DirFS<'a, T>
    where T: DirFSInterface<'a>
{
    implementor: T,
    parent_fs: ParentDirFS<'a>,
}

impl<'a, T> DirFS<'a, T>
    where T: DirFSInterface<'a>
{
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
            if reply.add(attr.ino, (offset + 1) as i64, attr.kind, DIR_NAME_SELF) {
                reply.ok();
                return;
            }

            offset += 1;
        }

        if offset == 1 {
            let attr = self.parent_fs.attr();
            if reply.add(attr.ino, (offset + 1) as i64, attr.kind, DIR_NAME_PARENT) {
                reply.ok();
                return;
            }

            offset += 1;
        }

        for (i, (attr, name)) in self.implementor.readdir_files().enumerate().skip((offset - 2) as usize) {
            let offset = i + 2;

            if reply.add(attr.ino, (offset + 1) as i64, attr.kind, name) {
                reply.ok();
                return;
            }
        }

        reply.ok();
    }
}

pub trait DirFSInterface<'a> {
    fn inode(&self) -> u64;
    fn attr(&self) -> FileAttr;
    fn name(&self) -> &str;

    fn fs_from_file_name(&self, name: &std::ffi::OsStr) -> Option<Box<dyn VirtualFile + 'a>> {
        if name == DIR_NAME_SELF {
            panic!("recursion");
        } else {
            None
        }
    }

    fn fs_from_inode(&self, inode: u64) -> Option<Box<dyn VirtualFile + 'a>> {
        if inode == self.attr().ino {
            panic!("recursion");
        } else {
            None
        }
    }

    fn readdir_files(&self) -> impl Iterator<Item = (FileAttr, &str)>;
}

impl<'a, T> Filesystem for DirFS<'a, T>
    where T: DirFSInterface<'a>
{
    fn lookup(&mut self, _req: &Request<'_>, parent: u64, name: &std::ffi::OsStr, reply: ReplyEntry) {
        if parent == self.implementor.inode() {
            if name == DIR_NAME_SELF {
                reply.entry(&DEFAULT_TTL, &self.attr(), 0);
            } else {
                let Some(file) = self.implementor.fs_from_file_name(name)
                    else { reply.error(libc::ENOENT); return; };

                reply.entry(&DEFAULT_TTL, &file.attr(), 0);
            }
        } else {
            let Some(mut file) = self.implementor.fs_from_inode(parent)
                else { reply.error(libc::ENOENT); return; };

            file.lookup(_req, parent, name, reply);
        }
    }

    fn getattr(&mut self, _req: &Request<'_>, ino: u64, fh: Option<u64>, reply: ReplyAttr) {
        if ino == PROC_DIR_INODE {
            reply.attr(&DEFAULT_TTL, &self.attr());
        } else {
            let Some(mut file) = self.implementor.fs_from_inode(ino)
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
        if ino == PROC_DIR_INODE {
            reply.error(libc::EISDIR);
        } else {
            let Some(mut file) = self.implementor.fs_from_inode(ino)
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
        if ino == PROC_DIR_INODE {
            reply.error(libc::EISDIR);
        } else {
            let Some(mut file) = self.implementor.fs_from_inode(ino)
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
        if ino == PROC_DIR_INODE {
            self._readdir(_req, ino, fh, offset, reply);
        } else {
            let Some(mut file) = self.implementor.fs_from_inode(ino)
                else { reply.error(libc::ENOENT); return; };

            file.readdir(_req, ino, fh, offset, reply);
        }
    }
}

impl<'a, T> VirtualFile for DirFS<'a, T>
    where T: DirFSInterface<'a>
{
    fn inode(&self) -> u64 {
        self.implementor.inode()
    }

    fn attr(&self) -> FileAttr {
        self.implementor.attr()
    }

    fn name(&self) -> &str {
        self.implementor.name()
    }
}