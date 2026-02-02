use fuser::*;
use crate::filesystem::utils::*;
use crate::ProcessStats;

pub struct CgroupFileFS<'a> {
    pub pid: sysinfo::Pid,
    pub stats: &'a ProcessStats,
}

impl CgroupFileFS<'_> {
    pub const NAME: &'static str = "cgroup";
    pub const INODE_OFFSET: u64 = 2;

    fn _read(&self) -> anyhow::Result<String> {
        use hcbs_utils::prelude::*;

        get_pid_cgroup(self.pid.as_u32())
            .map(|mut str| { str += "\n"; str })
    }

    fn _write(&self, cgroup: &str) -> anyhow::Result<()> {
        use hcbs_utils::prelude::*;

        if !get_sched_policy(self.pid.as_u32())?.is_other() {
            return Err(anyhow::format_err!("Only SCHED_OTHER processes are allowed to migrate."))
        }

        println!("{:?} {}", cgroup.as_bytes(), cgroup_abs_path(cgroup));

        assign_pid_to_cgroup(cgroup, self.pid.as_u32())
    }
}

impl VirtualFS for CgroupFileFS<'_> { }

impl Filesystem for CgroupFileFS<'_> {
    fn lookup(&mut self, _req: &Request<'_>, _parent: u64, _name: &std::ffi::OsStr, reply: ReplyEntry) {
        reply.entry(&DEFAULT_TTL, &self.attr(), 0);
    }

    fn getattr(&mut self, _req: &Request<'_>, _ino: u64, _fh: Option<u64>, reply: ReplyAttr) {
        reply.attr(&DEFAULT_TTL, &self.attr());
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
        let Ok(read_data) = self._read()
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

        match self._write(data) {
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

impl VirtualFile for CgroupFileFS<'_> {
    fn inode(&self) -> u64 {
        pid_to_dir_inode(self.pid) + Self::INODE_OFFSET
    }

    fn attr(&self) -> FileAttr {
        let inode = pid_to_dir_inode(self.pid);
        let read_data = self._read().unwrap_or(String::with_capacity(0));

        FileAttr {
            ino: inode + Self::INODE_OFFSET,
            size: read_data.len() as u64,
            blocks: u64::div_ceil(read_data.len() as u64, 512),
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