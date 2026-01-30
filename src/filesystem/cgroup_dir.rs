use fuser::*;
use crate::filesystem::utils::*;

mod create_cgroup_file;
mod update_cgroup_file;
mod delete_cgroup_file;

use create_cgroup_file::*;
use update_cgroup_file::*;
use delete_cgroup_file::*;

#[derive(Debug, Clone)]
pub struct CgroupDirFS<'a> {
    pub parent_fs: ParentDirFS<'a>,
}

impl<'a> CgroupDirFS<'a> {
    pub const NAME: &'static str = "cgroup";
}

impl<'a> DirFSInterface<'a> for CgroupDirFS<'a> {
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

    fn fs_from_file_name(&self, name: &std::ffi::OsStr) -> Option<Box<dyn VirtualFile + 'a>> {
        todo!()
    }

    fn fs_from_inode(&self, inode: u64) -> Option<Box<dyn VirtualFile + 'a>> {
        todo!()
    }

    fn readdir_files(&self) -> impl Iterator<Item = (FileAttr, &str)> {
        let create_file = CreateCgroupFileFS { parent_fs: ParentDirFS::new(self) };
        let delete_file = DeleteCgroupFileFS { parent_fs: ParentDirFS::new(self) };
        let update_file = UpdateCgroupFileFS { parent_fs: ParentDirFS::new(self) };

        [

        ]
    }
}