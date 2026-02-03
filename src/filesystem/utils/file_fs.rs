use fuser::*;
use super::*;

pub trait FileFSInterface: VirtualFile {
    fn read_size(&self) -> anyhow::Result<usize>;
    fn read_data(&self) -> anyhow::Result<&str>;
    fn write_data(&mut self, data: &str) -> anyhow::Result<()>;
}

#[derive(Debug)]
pub struct FileFS<T>
    where T: FileFSInterface
{
    implementor: T,
}

impl<T> FileFS<T>
    where T: FileFSInterface
{
    pub fn new(implementor: T) -> Self {
        Self { implementor }
    }
}

impl<T> VirtualFS for FileFS<T>
    where T: FileFSInterface { }

impl<T> Filesystem for FileFS<T>
    where T: FileFSInterface
{
    fn lookup(&mut self, _req: &Request<'_>, _parent: u64, _name: &std::ffi::OsStr, reply: ReplyEntry) {
        reply.entry(&DEFAULT_TTL, &self.attr(), 0);
    }

    fn getattr(&mut self, _req: &Request<'_>, _ino: u64, _fh: Option<u64>, reply: ReplyAttr) {
        reply.attr(&DEFAULT_TTL, &self.attr());
    }

    fn setattr(
        &mut self,
        _req: &Request<'_>,
        _ino: u64,
        _mode: Option<u32>,
        _uid: Option<u32>,
        _gid: Option<u32>,
        _size: Option<u64>,
        _atime: Option<TimeOrNow>,
        _mtime: Option<TimeOrNow>,
        _ctime: Option<std::time::SystemTime>,
        _fh: Option<u64>,
        _crtime: Option<std::time::SystemTime>,
        _chgtime: Option<std::time::SystemTime>,
        _bkuptime: Option<std::time::SystemTime>,
        _flags: Option<u32>,
        reply: ReplyAttr,
    ) {
        reply.attr(&DEFAULT_TTL, &self.attr());
    }

    fn read(
        &mut self,
        _req: &Request<'_>,
        _ino: u64,
        _fh: u64,
        offset: i64,
        size: u32,
        _flags: i32,
        _lock_owner: Option<u64>,
        reply: ReplyData,
    ) {
        let Ok(read_data) = self.implementor.read_data()
            else { reply.error(libc::EIO); return; };

        let read_data = read_data.as_bytes();
        let offset = offset as usize;
        let size = size as usize;

        let data =
            if offset < read_data.len() {
                &read_data[offset .. usize::min(read_data.len(), offset + size)]
            } else {
                &[]
            };

        reply.data(data);
    }

    fn write(
        &mut self,
        _req: &Request<'_>,
        _ino: u64,
        _fh: u64,
        offset: i64,
        data: &[u8],
        _write_flags: u32,
        _flags: i32,
        _lock_owner: Option<u64>,
        reply: ReplyWrite,
    ) {
        if offset != 0 {
            reply.error(libc::EINVAL);
            return;
        }

        let size = data.len();
        let Ok(data) = std::str::from_utf8(data).map(|str| str.trim())
            else { reply.error(libc::EIO); return; };

        match self.implementor.write_data(data) {
            Ok(()) => {
                reply.written(size as u32);
            },
            Err(err) => {
                debug!("Write error for {}: {}", self.name(), err);
                reply.error(libc::EACCES);
            },
        }
    }

    fn readdir(
        &mut self,
        _req: &Request<'_>,
        _ino: u64,
        _fh: u64,
        _offset: i64,
        reply: ReplyDirectory,
    ) {
        reply.error(libc::ENOTDIR);
    }
}

impl<T> VirtualFile for FileFS<T>
    where T: FileFSInterface + VirtualFile
{
    fn inode(&self) -> u64 {
        self.implementor.inode()
    }

    fn attr(&self) -> FileAttr {
        let mut attr = self.implementor.attr();
        attr.size = self.implementor.read_size().unwrap_or(0) as u64;
        attr.blksize = 512;
        attr.blocks = u64::div_ceil(attr.size, attr.blksize as u64);
        attr
    }

    fn name(&self) -> &str {
        self.implementor.name()
    }
}