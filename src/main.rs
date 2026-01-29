use hcbs_manager::prelude::*;

#[macro_use]
extern crate log;

#[derive(Debug, Clone)]
pub struct ProcessData {
    pid: sysinfo::Pid,
    uid: sysinfo::Uid,
    gid: sysinfo::Gid,
}

fn main() -> anyhow::Result<()> {
    // Debug Logging
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    HCBSController::new().mount()?;

    Ok(())
}
