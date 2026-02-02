use fuser::*;
use crate::filesystem::utils::*;

mod create_cgroup_file;
mod update_cgroup_file;
mod delete_cgroup_file;

use create_cgroup_file::*;
use update_cgroup_file::*;
use delete_cgroup_file::*;

#[derive(Debug, Clone)]
pub struct CgroupDirFS {

}

impl CgroupDirFS {
    pub const NAME: &'static str = "cgroup";

    pub fn new<'a>(parent_fs: ParentDirFS<'a>) -> DirFS<'a, Self> {
        DirFS {
            implementor: Self { },
            parent_fs,
        }
    }
}

impl DirFSInterface for CgroupDirFS {
    fn fs_from_file_name<'a>(&'a self, name: &std::ffi::OsStr) -> Option<Box<dyn VirtualFS + 'a>> {
        match name.to_str().unwrap() {
            CreateCgroupFileFS::NAME => Some(Box::new(CreateCgroupFileFS { })),
            DeleteCgroupFileFS::NAME => Some(Box::new(DeleteCgroupFileFS { })),
            UpdateCgroupFileFS::NAME => Some(Box::new(UpdateCgroupFileFS { })),
            _ => None,
        }
    }

    fn fs_from_inode<'a>(&'a self, inode: u64) -> Option<Box<dyn VirtualFS + 'a>> {
        match inode {
            CreateCgroupFileFS::INODE => Some(Box::new(CreateCgroupFileFS { })),
            DeleteCgroupFileFS::INODE => Some(Box::new(DeleteCgroupFileFS { })),
            UpdateCgroupFileFS::INODE => Some(Box::new(UpdateCgroupFileFS { })),
            _ => None,
        }
    }

    fn readdir_files<'a>(&'a self) -> impl Iterator<Item = Box<dyn VirtualFS + 'a>> {
        let files: [Box<dyn VirtualFS>; _] = [
            Box::new(CreateCgroupFileFS { }),
            Box::new(DeleteCgroupFileFS { }),
            Box::new(UpdateCgroupFileFS { })
        ];

        files.into_iter()
    }
}

impl VirtualFile for CgroupDirFS {
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