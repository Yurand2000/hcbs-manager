use fuser::*;
use crate::filesystem::utils::*;
use crate::ProcessStats;

pub struct SchedPolicyFileFS<'a> {
    pub pid: sysinfo::Pid,
    pub stats: &'a ProcessStats,
}

impl SchedPolicyFileFS<'_> {
    pub const NAME: &'static str = "sched_policy";
    pub const INODE_OFFSET: u64 = 3;
}

impl Filesystem for SchedPolicyFileFS<'_> {
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

impl VirtualFile for SchedPolicyFileFS<'_> {
    fn inode(&self) -> u64 {
        pid_to_dir_inode(self.pid) + Self::INODE_OFFSET
    }

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