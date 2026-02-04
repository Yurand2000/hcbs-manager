use hcbs_utils::prelude::*;

pub mod cgroup;
pub mod proc;

use cgroup::*;
use proc::*;

pub use cgroup::Reservation;

#[derive(Debug)]
pub struct HCBSManager {
    cgroups: CgroupManager,
    procs: ProcManager,
}

impl HCBSManager {
    pub fn new(reset_on_exit: bool) -> Self {
        Self {
            cgroups: CgroupManager::new(),
            procs: ProcManager::new(reset_on_exit),
        }
    }

    pub fn create_cgroup(&mut self, name: &str, request: Reservation) -> anyhow::Result<()> {
        self.cgroups.create_cgroup(name, request)
    }

    pub fn update_cgroup(&mut self, name: &str, request: Reservation) -> anyhow::Result<()> {
        self.cgroups.update_cgroup(name, request)
    }

    pub fn destroy_cgroup(&mut self, name: &str) -> anyhow::Result<()> {
        self.cgroups.destroy_cgroup(name)
    }

    pub fn is_managed_cgroup(&self, name: &str) -> bool {
        self.cgroups.is_managed_cgroup(name)
    }

    pub fn assign_cgroup_to_process(&mut self, pid: Pid, cgroup: &str) -> anyhow::Result<()> {
        self.procs.assign_cgroup_to_process(&self.cgroups, pid, cgroup)
    }

    pub fn set_process_sched_policy(&mut self, pid: Pid, policy: SchedPolicy) -> anyhow::Result<()> {
        self.procs.set_process_sched_policy(&self.cgroups, pid, policy)
    }
}