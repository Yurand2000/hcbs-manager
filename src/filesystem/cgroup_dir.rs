use fuser::*;
use crate::filesystem::utils::*;

mod create_cgroup_file;
mod update_cgroup_file;
mod delete_cgroup_file;

use create_cgroup_file::*;
use update_cgroup_file::*;
use delete_cgroup_file::*;

#[derive(Debug)]
pub struct CgroupDirFS<'a: 'b, 'b> {
    root_fs: &'b mut super::RootFS<'a>,
}

impl<'a: 'b, 'b> CgroupDirFS<'a, 'b> {
    pub const NAME: &'static str = "cgroup";

    pub fn new(root_fs: &'b mut super::RootFS<'a>) -> DirFS<Self> {
        DirFS::new( Self { root_fs } )
    }

    fn cgroup_manager(&mut self) -> &mut crate::manager::CgroupManager {
        self.root_fs.cgroup_manager
    }
}

impl DirFSInterface for CgroupDirFS<'_, '_> {
    fn parent_attr(&self) -> Option<FileAttr> {
        Some(self.root_fs.attr())
    }

    fn fs_from_file_name<'a>(&'a mut self, name: &std::ffi::OsStr) -> Option<Box<dyn VirtualFS + 'a>> {
        match name.to_str().unwrap() {
            CreateCgroupFileFS::NAME => Some(Box::new(CreateCgroupFileFS::new(self.cgroup_manager()))),
            DeleteCgroupFileFS::NAME => Some(Box::new(DeleteCgroupFileFS::new(self.cgroup_manager()))),
            UpdateCgroupFileFS::NAME => Some(Box::new(UpdateCgroupFileFS::new(self.cgroup_manager()))),
            _ => None,
        }
    }

    fn fs_from_inode<'a>(&'a mut self, inode: u64) -> Option<Box<dyn VirtualFS + 'a>> {
        match inode {
            CreateCgroupFileFS::INODE => Some(Box::new(CreateCgroupFileFS::new(self.cgroup_manager()))),
            DeleteCgroupFileFS::INODE => Some(Box::new(DeleteCgroupFileFS::new(self.cgroup_manager()))),
            UpdateCgroupFileFS::INODE => Some(Box::new(UpdateCgroupFileFS::new(self.cgroup_manager()))),
            _ => None,
        }
    }

    fn fs_inodes_in_dir(&self) -> impl Iterator<Item = u64> {
        [
            CreateCgroupFileFS::INODE,
            DeleteCgroupFileFS::INODE,
            UpdateCgroupFileFS::INODE,
        ].into_iter()
    }
}

impl VirtualFile for CgroupDirFS<'_, '_> {
    fn inode(&self) -> u64 {
        CGROUP_DIR_INODE
    }

    fn attr(&self) -> FileAttr {
        FileAttr {
            ino: CGROUP_DIR_INODE,
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
        Self::NAME
    }
}