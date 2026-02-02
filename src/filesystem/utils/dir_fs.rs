use fuser::*;
use super::*;

pub trait DirFSInterface: VirtualFile {
    fn fs_from_file_name<'a>(&'a mut self, _name: &std::ffi::OsStr) -> Option<Box<dyn VirtualFS + 'a>> { None }
    fn fs_from_inode<'a>(&'a mut self, _inode: u64) -> Option<Box<dyn VirtualFS + 'a>> { None }
    fn readdir_files<'a>(&'a mut self) -> impl Iterator<Item = Box<dyn VirtualFS + 'a>>;
}

#[derive(Debug, Clone)]
pub struct DirFS<'a, T>
    where T: DirFSInterface
{
    pub implementor: T,
    pub parent_fs: ParentDirFS<'a>,
}

impl<'a, T> DirFS<'a, T>
    where T: DirFSInterface
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

        for (i, file) in self.implementor.readdir_files().enumerate().skip((offset - 2) as usize) {
            let offset = i + 2;
            let attr = file.attr();
            let name = file.name();

            if reply.add(attr.ino, (offset + 1) as i64, attr.kind, name) {
                reply.ok();
                return;
            }
        }

        reply.ok();
    }
}

impl<'a, T> VirtualFS for DirFS<'a, T>
    where T: DirFSInterface { }

impl<'a, T> Filesystem for DirFS<'a, T>
    where T: DirFSInterface
{
    fn lookup(&mut self, _req: &Request<'_>, parent: u64, name: &std::ffi::OsStr, reply: ReplyEntry) {
        if parent == self.implementor.inode() {
            if name == DIR_NAME_SELF {
                reply.entry(&DEFAULT_TTL, &self.attr(), 0);
            } else if name == DIR_NAME_PARENT {
                reply.entry(&DEFAULT_TTL, &self.parent_fs.attr(), 0);
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
        if ino == self.inode() {
            reply.attr(&DEFAULT_TTL, &self.attr());
        } else {
            let Some(mut file) = self.implementor.fs_from_inode(ino)
                else { reply.error(libc::ENOENT); return; };

            file.getattr(_req, ino, fh, reply);
        }
    }

    fn setattr(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        mode: Option<u32>,
        uid: Option<u32>,
        gid: Option<u32>,
        size: Option<u64>,
        _atime: Option<TimeOrNow>,
        _mtime: Option<TimeOrNow>,
        _ctime: Option<std::time::SystemTime>,
        fh: Option<u64>,
        _crtime: Option<std::time::SystemTime>,
        _chgtime: Option<std::time::SystemTime>,
        _bkuptime: Option<std::time::SystemTime>,
        flags: Option<u32>,
        reply: ReplyAttr,
    ) {
        if ino == self.inode() {
            reply.error(libc::ENOSYS);
        } else {
            let Some(mut file) = self.implementor.fs_from_inode(ino)
                else { reply.error(libc::ENOENT); return; };

            file.setattr(_req, ino, mode, uid, gid, size, _atime, _mtime, _ctime, fh, _crtime, _chgtime, _bkuptime, flags, reply);
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
        if ino == self.inode() {
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
        if ino == self.inode() {
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
        if ino == self.inode() {
            self._readdir(_req, ino, fh, offset, reply);
        } else {
            let Some(mut file) = self.implementor.fs_from_inode(ino)
                else { reply.error(libc::ENOENT); return; };

            file.readdir(_req, ino, fh, offset, reply);
        }
    }
}

impl<'a, T> VirtualFile for DirFS<'a, T>
    where T: DirFSInterface + VirtualFile
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

#[derive(Debug, Clone)]
pub struct DirNoParentFS<T>
    where T: DirFSInterface
{
    pub implementor: T,
}

impl<T> DirNoParentFS<T>
    where T: DirFSInterface
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

        for (i, file) in self.implementor.readdir_files().enumerate().skip((offset - 1) as usize) {
            let offset = i + 1;
            let attr = file.attr();
            let name = file.name();

            if reply.add(attr.ino, (offset + 1) as i64, attr.kind, name) {
                reply.ok();
                return;
            }
        }

        reply.ok();
    }
}

impl<T> VirtualFS for DirNoParentFS<T>
    where T: DirFSInterface { }

impl<T> Filesystem for DirNoParentFS<T>
    where T: DirFSInterface
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
        if ino == self.inode() {
            reply.attr(&DEFAULT_TTL, &self.attr());
        } else {
            let Some(mut file) = self.implementor.fs_from_inode(ino)
                else { reply.error(libc::ENOENT); return; };

            file.getattr(_req, ino, fh, reply);
        }
    }

    fn setattr(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        mode: Option<u32>,
        uid: Option<u32>,
        gid: Option<u32>,
        size: Option<u64>,
        _atime: Option<TimeOrNow>,
        _mtime: Option<TimeOrNow>,
        _ctime: Option<std::time::SystemTime>,
        fh: Option<u64>,
        _crtime: Option<std::time::SystemTime>,
        _chgtime: Option<std::time::SystemTime>,
        _bkuptime: Option<std::time::SystemTime>,
        flags: Option<u32>,
        reply: ReplyAttr,
    ) {
        if ino == self.inode() {
            reply.error(libc::ENOSYS);
        } else {
            let Some(mut file) = self.implementor.fs_from_inode(ino)
                else { reply.error(libc::ENOENT); return; };

            file.setattr(_req, ino, mode, uid, gid, size, _atime, _mtime, _ctime, fh, _crtime, _chgtime, _bkuptime, flags, reply);
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
        if ino == self.inode() {
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
        if ino == self.inode() {
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
        if ino == self.inode() {
            self._readdir(_req, ino, fh, offset, reply);
        } else {
            let Some(mut file) = self.implementor.fs_from_inode(ino)
                else { reply.error(libc::ENOENT); return; };

            file.readdir(_req, ino, fh, offset, reply);
        }
    }
}

impl<T> VirtualFile for DirNoParentFS<T>
    where T: DirFSInterface + VirtualFile
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