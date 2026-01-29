use fuser::*;

pub struct FileInodeFnParams {
    inode: u64,
}

pub struct FileAttrFnParams<'a> {
    inode: u64,
    pid: sysinfo::Pid,
    stats: &'a super::ProcessStats,
}

pub struct FileReadFnParams<'a> {
    inode: u64,
    pid: sysinfo::Pid,
    stats: &'a super::ProcessStats,
    offset: i64,
    size: u32,
}

pub struct FileController {
    name: fn() -> &'static std::ffi::OsStr,
    inode: fn(&super::HCBSController, FileInodeFnParams) -> u64,
    file_attr: fn(&super::HCBSController, FileAttrFnParams) -> FileAttr,
    read: fn(FileReadFnParams) -> Option<&[u8]>,
}

impl super::HCBSController {
    const ROOT_INODE: u64 = 1;
    const RESERVED_INODE_SHIFT: u64 = 3;
    const RESERVED_INODE_MASK: u64 = (1 << Self::RESERVED_INODE_SHIFT) - 1;
    const INODE_DIR_OFFSET: u64 = 0;
    const UNKNOWN_TIME: std::time::SystemTime = std::time::UNIX_EPOCH;
    const DEFAULT_TTL: std::time::Duration = std::time::Duration::from_millis(1);
    const DEFAULT_DIR_ATTR: FileAttr = FileAttr {
        ino: 0,
        size: 0,
        blocks: 0,
        atime: Self::UNKNOWN_TIME,
        mtime: Self::UNKNOWN_TIME,
        ctime: Self::UNKNOWN_TIME,
        crtime: Self::UNKNOWN_TIME,
        kind: FileType::Directory,
        perm: 0o775,
        nlink: 1,
        uid: 0,
        gid: 0,
        rdev: 0,
        blksize: 512,
        flags: 0,
    };
    const DEFAULT_FILE_ATTR: FileAttr = FileAttr {
        ino: 0,
        size: 0,
        blocks: 0,
        atime: Self::UNKNOWN_TIME,
        mtime: Self::UNKNOWN_TIME,
        ctime: Self::UNKNOWN_TIME,
        crtime: Self::UNKNOWN_TIME,
        kind: FileType::RegularFile,
        perm: 0o664,
        nlink: 1,
        uid: 0,
        gid: 0,
        rdev: 0,
        blksize: 512,
        flags: 0,
    };

    const FILES: [FileController; 4] = [
        FileController {
            name: || &std::ffi::OsStr::new("."),
            inode: |_, p| p.inode,
            file_attr: |_, p| {
                let mut attr = Self::DEFAULT_DIR_ATTR;
                attr.ino = p.inode;
                attr.uid = *p.stats.uid;
                attr.gid = *p.stats.gid;
                attr.atime = p.stats.crtime;
                attr.mtime = p.stats.crtime;
                attr.ctime = p.stats.crtime;
                attr.crtime = p.stats.crtime;
                attr
            },
            read: |_| None,
        },
        FileController {
            name: || &std::ffi::OsStr::new(".."),
            inode: |_, _| Self::ROOT_INODE,
            file_attr: |ctrl, _| ctrl.root_dir_attr(),
            read: |_| None,
        },
        FileController {
            name: || &std::ffi::OsStr::new("cgroup"),
            inode: |_, p| p.inode + 1,
            file_attr: |_, p| {
                let mut attr = Self::DEFAULT_FILE_ATTR;
                attr.ino = p.inode + 1;
                attr.uid = *p.stats.uid;
                attr.gid = *p.stats.gid;
                attr.crtime = p.stats.crtime;
                attr.blocks = 1;
                attr.size = 8;
                attr
            },
            read: |p| {
                let data = "unknown\n".as_bytes();
                let offset = p.offset as usize;
                let size = p.size as usize;

                if offset > data.len() {
                    Some(&data[data.len() - 1 .. ])
                } else {
                    Some(&data[offset .. data.len().min(offset + size)])
                }
            },
        },
        FileController {
            name: || &std::ffi::OsStr::new("sched_policy"),
            inode: |_, p| p.inode + 2,
            file_attr: |_, p| {
                let mut attr = Self::DEFAULT_FILE_ATTR;
                attr.ino = p.inode + 2;
                attr.uid = *p.stats.uid;
                attr.gid = *p.stats.gid;
                attr.crtime = p.stats.crtime;
                attr.blocks = 1;
                attr.size = 8;
                attr
            },
            read: |p| {
                let data = "unknown\n".as_bytes();
                let offset = p.offset as usize;
                let size = p.size as usize;

                if offset > data.len() {
                    Some(&data[data.len() - 1 .. ])
                } else {
                    Some(&data[offset .. data.len().min(offset + size)])
                }
            },
        },
    ];

    fn root_dir_attr(&self) -> FileAttr {
        let mut attr = Self::DEFAULT_DIR_ATTR;
        attr.ino = Self::ROOT_INODE;
        attr.kind = FileType::Directory;
        attr.nlink = 2;

        attr
    }

    fn inode_to_pid(inode: u64) -> Option<sysinfo::Pid> {
        let Ok(inode) = u32::try_from(inode >> Self::RESERVED_INODE_SHIFT) else { return None; };
        Some(sysinfo::Pid::from_u32(inode))
    }

    fn pid_to_dir_inode(pid: sysinfo::Pid) -> u64 {
        (pid.as_u32() as u64) << Self::RESERVED_INODE_SHIFT + Self::INODE_DIR_OFFSET
    }

    fn process_from_pid(&self, pid: sysinfo::Pid) -> Option<&super::ProcessStats> {
        let stats = self.active_procs.get(&pid)?;

        Some(stats)
    }

    fn process_from_inode(&self, inode: u64) -> Option<(sysinfo::Pid, &super::ProcessStats)> {
        let pid = Self::inode_to_pid(inode)?;
        let stats = self.active_procs.get(&pid)?;

        Some((pid, stats))
    }

    fn file_from_inode(&self, inode: u64) -> Option<&FileController> {
        let pid_inode = inode & !Self::RESERVED_INODE_MASK;

        Self::FILES.iter().find(|f| {
            let params = FileInodeFnParams { inode: pid_inode };
            (f.inode)(self, params) == inode
        })
    }

    fn file_from_name(&self, name: &std::ffi::OsStr) -> Option<&FileController> {
        Self::FILES.iter().find(|file| (file.name)() == name)
    }
}

impl Filesystem for super::HCBSController {
    fn lookup(&mut self, _req: &Request<'_>, parent: u64, name: &std::ffi::OsStr, reply: ReplyEntry) {
        debug!("Lookup: parent {parent}, name {name:?}");
        self.update_active_processes();

        let (inode, pid, proc, file) =
            if parent == Self::ROOT_INODE {
                let Some(pid) = name.to_str().and_then(|str| str.parse::<u32>().ok())
                    else { reply.error(libc::ENOENT); return; };
                let pid = sysinfo::Pid::from_u32(pid);

                let Some(proc) = self.process_from_pid(pid)
                    else { reply.error(libc::ENOENT); return; };

                let inode = Self::pid_to_dir_inode(pid);
                let Some(file) = self.file_from_inode(inode)
                    else { reply.error(libc::ENOENT); return; };

                (inode, pid, proc, file)
            } else {
                let Some((pid, proc)) = self.process_from_inode(parent)
                    else { reply.error(libc::ENOENT); return; };

                let inode = Self::pid_to_dir_inode(pid);
                let Some(file) = self.file_from_name(name)
                    else { reply.error(libc::ENOENT); return; };

                (inode, pid, proc, file)
            };

        let params = FileAttrFnParams { inode, pid, stats: proc };
        reply.entry(&Self::DEFAULT_TTL, &(file.file_attr)(self, params), 0);
    }

    fn getattr(&mut self, _req: &Request<'_>, ino: u64, fh: Option<u64>, reply: ReplyAttr) {
        debug!("Getattr: inode {ino}");
        self.update_active_processes();

        if ino == Self::ROOT_INODE {
            reply.attr(&Self::DEFAULT_TTL, &self.root_dir_attr());
            return;
        }

        let Some((pid, proc)) = self.process_from_inode(ino)
            else { reply.error(libc::ENOENT); return; };

        let Some(file) = self.file_from_inode(ino)
            else { reply.error(libc::ENOENT); return; };

        let params = FileAttrFnParams { inode: ino, pid, stats: proc };
        reply.attr(&Self::DEFAULT_TTL, &(file.file_attr)(self, params));
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
        debug!("Read: inode {ino}, offset {offset}, size {size}");
        self.update_active_processes();

        let Some((pid, proc)) = self.process_from_inode(ino)
            else { reply.error(libc::ENOENT); return; };

        let Some(file) = self.file_from_inode(ino)
            else { reply.error(libc::ENOENT); return; };

        debug!("Read on file {:?} for pid {}", (file.name)(), pid);

        let params = FileReadFnParams { inode: ino, pid, stats: proc, offset, size };
        let Some(out_data) = (file.read)(params)
            else { reply.error(libc::ENOENT); return; };

        reply.data(out_data);
    }

    fn readdir(
            &mut self,
            _req: &Request<'_>,
            ino: u64,
            fh: u64,
            offset: i64,
            mut reply: ReplyDirectory,
        ) {
        debug!("Readdir: inode {ino}, offset {offset}");
        self.update_active_processes();

        if ino == Self::ROOT_INODE {
            for (offset, (pid, _)) in self.active_procs.iter().enumerate().skip(offset as usize) {
                let inode = Self::pid_to_dir_inode(*pid);

                debug!("Readdir reply: inode {inode}, offset {offset}, pid {pid}");
                if reply.add(inode, (offset + 1) as i64, FileType::Directory, format!("{pid}")) {
                    break;
                }
            }

            reply.ok();
            return;
        }

        if ino & Self::RESERVED_INODE_MASK != 0 {
            reply.error(libc::ENOTDIR);
            return;
        }

        let Some((pid, proc)) = self.process_from_inode(ino)
            else { reply.error(libc::ENOENT); return; };

        for (offset, file) in Self::FILES.iter().enumerate().skip(offset as usize) {
            let params = FileAttrFnParams { inode: ino, pid, stats: proc };
            let attr = (file.file_attr)(self, params);

            if reply.add(attr.ino, (offset + 1) as i64, attr.kind, (file.name)()) {
                break;
            }
        }

        reply.ok();
    }
}