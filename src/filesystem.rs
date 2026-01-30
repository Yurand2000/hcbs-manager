use fuser::*;

mod process;
mod root;
mod utils;

pub trait VirtualFile: Filesystem {
    fn attr(&self) -> FileAttr;
    fn name(&self) -> &str;
}

impl Filesystem for super::HCBSController {
    fn lookup(&mut self, _req: &Request<'_>, parent: u64, name: &std::ffi::OsStr, reply: ReplyEntry) {
        self.update_active_processes();

        root::RootFS { active_procs: &self.active_procs }
            .lookup(_req, parent, name, reply);
    }

    fn getattr(&mut self, _req: &Request<'_>, ino: u64, fh: Option<u64>, reply: ReplyAttr) {
        self.update_active_processes();

        root::RootFS { active_procs: &self.active_procs }
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

        root::RootFS { active_procs: &self.active_procs }
            .read(_req, ino, fh, offset, size, flags, lock_owner, reply);
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

        root::RootFS { active_procs: &self.active_procs }
            .readdir(_req, ino, fh, offset, reply);
    }
}