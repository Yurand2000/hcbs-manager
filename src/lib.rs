use std::collections::HashMap;

use hcbs_utils::prelude::*;

#[macro_use]
extern crate log;

mod filesystem;
mod manager;
mod utils;

pub mod prelude {
    pub use super::{
        Controller,
    };
}

#[derive(Debug)]
pub struct Controller {
    mountpoint: &'static str,
    manager: manager::HCBSManager,
    process_info: ProcessInfo,
}

#[derive(Debug, Clone)]
pub struct ProcessStats {
    uid: sysinfo::Uid,
    gid: sysinfo::Gid,
    crtime: std::time::SystemTime,
}

impl Controller {
    const DEFAULT_MOUNT_POINT: &'static str = "/mnt/hcbs-manager";

    pub fn new(keep_on_exit: bool) -> Self {
        Self {
            mountpoint: Self::DEFAULT_MOUNT_POINT,
            manager: manager::HCBSManager::new(keep_on_exit),
            process_info: ProcessInfo::new(),
        }
    }

    pub fn mount(self) -> anyhow::Result<()> {
        let mountpoint = self.mountpoint;

        ctrlc::set_handler(move || {
            std::process::Command::new("umount")
                .arg(mountpoint)
                .output()
                .unwrap();
        })?;

        let _mountdir = utils::TempDir::new(mountpoint)?;

        fuser::mount2(
            self,
            mountpoint,
            &[
                fuser::MountOption::AllowOther,
                fuser::MountOption::AutoUnmount,
                fuser::MountOption::DefaultPermissions,
                fuser::MountOption::NoDev,
                fuser::MountOption::NoSuid,
                fuser::MountOption::RW,
            ]
        )?;

        Ok(())
    }

    pub fn update(&mut self) {
        let dead = self.process_info.update_active_processes();

        self.manager.update_managed_processes(dead);
    }
}

#[derive(Debug)]
struct ProcessInfo {
    sysinfo: sysinfo::System,
    active_procs: HashMap<sysinfo::Pid, ProcessStats>,
    last_update: std::time::Instant,
}

impl ProcessInfo {
    const UPDATE_DELTA: std::time::Duration = std::time::Duration::from_secs(1);

    pub fn new() -> Self {
        Self {
            sysinfo: sysinfo::System::new(),
            active_procs: HashMap::with_capacity(0),
            last_update: std::time::Instant::now() - Self::UPDATE_DELTA * 2,
        }
    }

    fn update_active_processes(&mut self) -> Box<dyn Iterator<Item = Pid> + '_> {
        use sysinfo::*;

        let now = std::time::Instant::now();
        if now - self.last_update <= Self::UPDATE_DELTA {
            return Box::new(std::iter::empty());
        }

        self.sysinfo.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing()
                .with_user(UpdateKind::Always));

        let (alive, dead): (Vec<_>, Vec<_>) =
            self.sysinfo.processes().iter()
                .partition(|(_, p)| p.exists());

        self.active_procs =
                alive.into_iter()
                .map(|(_, p)|
                (p.pid(), ProcessStats {
                    uid: p.user_id().unwrap().clone(),
                    gid: p.group_id().unwrap().clone(),
                    crtime: std::time::UNIX_EPOCH + std::time::Duration::from_secs(p.start_time()),
                }))
                .collect();

        self.last_update = std::time::Instant::now();

        Box::new(dead.into_iter().map(|(pid, _)| pid.as_u32()))
    }
}