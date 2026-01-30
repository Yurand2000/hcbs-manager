use fuser::*;
use crate::filesystem::{VirtualFile, utils::*};
use crate::ProcessStats;

pub struct CgroupFileFS<'a> {
    pub pid: sysinfo::Pid,
    pub stats: &'a ProcessStats,
}

impl CgroupFileFS<'_> {
    pub const NAME: &'static str = "cgroup";
    pub const INODE_OFFSET: u64 = 2;
}

impl Filesystem for CgroupFileFS<'_> {
    fn lookup(&mut self, _req: &Request<'_>, _parent: u64, _name: &std::ffi::OsStr, reply: ReplyEntry) {
        reply.entry(&DEFAULT_TTL, &self.attr(), 0);
    }

    fn getattr(&mut self, _req: &Request<'_>, _ino: u64, _fh: Option<u64>, reply: ReplyAttr) {
        reply.attr(&DEFAULT_TTL, &self.attr());
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

impl VirtualFile for CgroupFileFS<'_> {
    fn attr(&self) -> FileAttr {
        let inode = pid_to_dir_inode(self.pid);

        FileAttr {
            ino: inode + Self::INODE_OFFSET,
            size: 0,
            blocks: 0,
            atime: self.stats.crtime,
            mtime: self.stats.crtime,
            ctime: self.stats.crtime,
            crtime: self.stats.crtime,
            kind: FileType::RegularFile,
            perm: 0o664,
            nlink: 1,
            uid: *self.stats.uid,
            gid: *self.stats.gid,
            rdev: 0,
            blksize: 512,
            flags: 0,
        }
    }

    fn name(&self) -> &str {
        Self::NAME
    }
}