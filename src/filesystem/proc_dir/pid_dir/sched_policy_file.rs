use fuser::*;
use hcbs_utils::prelude::*;
use nom::sequence::delimited;
use crate::filesystem::utils::*;
use crate::ProcessStats;

pub struct SchedPolicyFileFS<'a> {
    pid: sysinfo::Pid,
    stats: &'a ProcessStats,
    policy: Option<(SchedPolicy, String)>,
    manager: &'a mut crate::manager::HCBSManager,
}

impl<'a> SchedPolicyFileFS<'a> {
    pub const NAME: &'static str = "sched_policy";
    pub const INODE_OFFSET: u64 = 3;

    pub fn new(pid_dir_fs: &'a mut super::PidDirFS<'_>) -> FileFS<Self> {
        let policy = get_sched_policy(pid_dir_fs.pid.as_u32())
            .map(|policy| {
                use SchedPolicy::*;

                let str = match policy {
                    OTHER { .. } => format!("SCHED_OTHER\n"),
                    BATCH { .. } => format!("SCHED_BATCH\n"),
                    IDLE => format!("SCHED_IDLE\n"),
                    FIFO(prio) => format!("SCHED_FIFO({prio})\n"),
                    RR(prio) => format!("SCHED_RR({prio})\n"),
                    DEADLINE { .. } => format!("SCHED_DEADLINE\n"),
                };

                (policy, str)
            }).ok();

        FileFS::new( Self {
            pid: pid_dir_fs.pid,
            stats: pid_dir_fs.stats,
            policy,
            manager: pid_dir_fs.manager,
        } )
    }

    fn parse_request(data: &str) -> Option<SchedPolicy> {
        use nom::Parser as _;
        use nom::branch::*;
        use nom::bytes::complete::*;
        use nom::combinator::*;

        alt((
            value(
                SchedPolicy::other(),
                tag("SCHED_OTHER"),
            ),
            map_res(
                (
                    tag("SCHED_FIFO"),
                    delimited(
                        tag("("),
                        crate::filesystem::utils::
                            parser::parse_u64,
                        tag(")")
                    )
                ),
                |(_, prio)| prio.try_into().map(|prio| SchedPolicy::FIFO(prio))
            ),
            map_res(
                (
                    tag("SCHED_RR"),
                    delimited(
                        tag("("),
                        crate::filesystem::utils::
                            parser::parse_u64,
                        tag(")")
                    )
                ),
                |(_, prio)| prio.try_into().map(|prio| SchedPolicy::RR(prio))
            )
        )).parse(data).map(|(_, policy)| policy).ok()
    }
}

impl FileFSInterface for SchedPolicyFileFS<'_> {
    fn read_size(&self) -> anyhow::Result<usize> {
        let Some((_, str)) = &self.policy
            else { anyhow::bail!("SchedPolicy not found") };

        Ok(str.len())
    }

    fn read_data(&self) -> anyhow::Result<&str> {
        let Some((_, str)) = &self.policy
            else { anyhow::bail!("SchedPolicy not found") };

        Ok(str.as_str())
    }

    fn write_data(&mut self, data: &str) -> anyhow::Result<()> {
        let Some(policy) = Self::parse_request(data)
            else { anyhow::bail!("Invalid request"); };

        self.manager.set_process_sched_policy(self.pid.as_u32(), policy)
    }
}

impl VirtualFile for SchedPolicyFileFS<'_> {
    fn inode(&self) -> u64 {
        pid_to_dir_inode(self.pid) + Self::INODE_OFFSET
    }

    fn attr(&self) -> FileAttr {
        FileAttr {
            ino: self.inode(),
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