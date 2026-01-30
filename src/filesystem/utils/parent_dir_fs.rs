use fuser::*;
use super::*;

#[derive(Debug, Clone)]
pub struct ParentDirFS<'a> {
    attr: FileAttr,
    name: &'a str,
}

impl<'a> ParentDirFS<'a> {
    pub fn new<T: VirtualFile>(dir: &'a T) -> Self {
        Self { attr: dir.attr(), name: dir.name() }
    }
}

impl Filesystem for ParentDirFS<'_> {
    fn lookup(&mut self, _req: &Request<'_>, _parent: u64, _name: &std::ffi::OsStr, reply: ReplyEntry) {
        reply.entry(&DEFAULT_TTL, &self.attr, 0);
    }

    fn getattr(&mut self, _req: &Request<'_>, _ino: u64, _fh: Option<u64>, reply: ReplyAttr) {
        reply.attr(&DEFAULT_TTL, &self.attr);
    }
}

impl VirtualFile for ParentDirFS<'_> {
    fn inode(&self) -> u64 {
        self.attr.ino
    }

    fn attr(&self) -> FileAttr {
        self.attr
    }

    fn name(&self) -> &str {
        &self.name
    }
}