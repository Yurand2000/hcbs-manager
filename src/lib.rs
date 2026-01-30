use std::collections::HashMap;

#[macro_use]
extern crate log;

mod filesystem;

pub mod prelude {
    pub use super::{
        HCBSController,
    };
}

#[derive(Debug)]
pub struct HCBSController {
    sysinfo: sysinfo::System,
    mountpoint: &'static str,
    active_procs: HashMap<sysinfo::Pid, ProcessStats>,
    last_update: std::time::Instant,
}

#[derive(Debug, Clone)]
pub struct ProcessStats {
    uid: sysinfo::Uid,
    gid: sysinfo::Gid,
    crtime: std::time::SystemTime,
}

impl HCBSController {
    pub fn new() -> Self {
        let mut ctrl = Self {
            sysinfo: sysinfo::System::new(),
            mountpoint: "/hcbs-manager",
            active_procs: HashMap::with_capacity(0),
            last_update: std::time::Instant::now(),
        };

        ctrl.update_active_processes();
        ctrl
    }

    pub fn mount(self) -> anyhow::Result<()> {
        let mountpoint = self.mountpoint;

        ctrlc::set_handler(move || {
            std::process::Command::new("umount")
                .arg(mountpoint)
                .output()
                .unwrap();
        })?;

        std::fs::create_dir(mountpoint)?;

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
                fuser::MountOption::NoExec,
                fuser::MountOption::Sync,
            ]
        )?;

        std::fs::remove_dir(mountpoint)?;

        Ok(())
    }

    fn update_active_processes(&mut self) {
        use sysinfo::*;

        const UPDATE_DELTA: std::time::Duration = std::time::Duration::from_secs(1);

        let now = std::time::Instant::now();
        if now - self.last_update <= UPDATE_DELTA {
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