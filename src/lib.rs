use std::collections::HashMap;

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

    pub fn new(reset_on_exit: bool) -> Self {
        Self {
            mountpoint: Self::DEFAULT_MOUNT_POINT,
            manager: manager::HCBSManager::new(reset_on_exit),
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
                // fuser::MountOption::NoExec,
                // fuser::MountOption::Sync,
            ]
        )?;

        Ok(())
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

    pub fn get_processes(&mut self) -> &mut HashMap<sysinfo::Pid, ProcessStats> {
        self.update_active_processes();

        &mut self.active_procs
    }

    fn update_active_processes(&mut self) {
        use sysinfo::*;

        let now = std::time::Instant::now();
        if now - self.last_update <= Self::UPDATE_DELTA {
            return;
        }

        self.sysinfo.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing()
                .with_user(UpdateKind::Always));

        self.active_procs =
            self.sysinfo.processes().iter()
                .filter(|(_, p)| p.exists())
                .map(|(_, p)|
                (p.pid(), ProcessStats {
                    uid: p.user_id().unwrap().clone(),
                    gid: p.group_id().unwrap().clone(),
                    crtime: std::time::UNIX_EPOCH + std::time::Duration::from_secs(p.start_time()),
                }))
                .collect();

        self.last_update = std::time::Instant::now();
    }
}