use fuser::*;
use crate::filesystem::utils::*;

mod create_cgroup_file;
mod update_cgroup_file;
mod delete_cgroup_file;

use create_cgroup_file::*;
use update_cgroup_file::*;
use delete_cgroup_file::*;

#[derive(Debug)]
pub struct CgroupDirFS<'a> {
    manager: &'a mut crate::manager::HCBSManager,
    root_fs_attr: FileAttr,
}

impl<'a> CgroupDirFS<'a> {
    pub const NAME: &'static str = "cgroup";

    pub fn new(root_fs: &'a mut super::RootFS<'_>) -> DirFS<Self> {
        let root_fs_attr = root_fs.attr();

        DirFS::new( Self {
            manager: root_fs.manager,
            root_fs_attr,
        } )
    }
}

impl DirFSInterface for CgroupDirFS<'_> {
    fn parent_attr(&self) -> Option<FileAttr> {
        Some(self.root_fs_attr)
    }

    fn fs_from_file_name<'a>(&'a mut self, name: &std::ffi::OsStr) -> Option<Box<dyn VirtualFS + 'a>> {
        match name.to_str().unwrap() {
            CreateCgroupFileFS::NAME => Some(Box::new(CreateCgroupFileFS::new(self))),
            DeleteCgroupFileFS::NAME => Some(Box::new(DeleteCgroupFileFS::new(self))),
            UpdateCgroupFileFS::NAME => Some(Box::new(UpdateCgroupFileFS::new(self))),
            _ => None,
        }
    }

    fn fs_from_inode<'a>(&'a mut self, inode: u64) -> Option<Box<dyn VirtualFS + 'a>> {
        match inode {
            CreateCgroupFileFS::INODE => Some(Box::new(CreateCgroupFileFS::new(self))),
            DeleteCgroupFileFS::INODE => Some(Box::new(DeleteCgroupFileFS::new(self))),
            UpdateCgroupFileFS::INODE => Some(Box::new(UpdateCgroupFileFS::new(self))),
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

impl VirtualFile for CgroupDirFS<'_> {
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