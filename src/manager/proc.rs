use std::collections::HashMap;

use hcbs_utils::prelude::*;

#[derive(Debug)]
pub struct ProcManager {
    procs: HashMap<Pid, ProcData>,
    keep_on_exit: bool,
}

#[derive(Debug)]
pub struct ProcData {
    original_cgroup: String,
}

impl ProcManager {
    pub fn new(keep_on_exit: bool) -> Self {
        Self { procs: HashMap::new(), keep_on_exit }
    }

    pub fn update_managed_processes<I>(&mut self, dead_procs: I)
        where I: Iterator<Item = Pid>,
    {
        for proc in dead_procs {
            self.procs.remove(&proc);
        }
    }

    pub fn assign_cgroup_to_process(&mut self, cgroups: &super::CgroupManager, pid: Pid, cgroup: &str) -> anyhow::Result<()> {
        if !cgroup_exists(cgroup) {
            anyhow::bail!("Cgroup \"{cgroup}\" does not exist");
        }

        if !cgroups.is_managed_cgroup(cgroup) {
            anyhow::bail!("Cgroup \"{cgroup}\" is not managed by this controller.");
        }

        self.get_managed_process(pid)?;

        if !get_sched_policy(pid)?.is_other() {
            anyhow::bail!("Only SCHED_OTHER processes are allowed to be moved between cgroups.");
        }

        assign_pid_to_cgroup(cgroup, pid)?;

        Ok(())
    }

    pub fn set_process_sched_policy(&mut self, cgroups: &super::CgroupManager, pid: Pid, policy: SchedPolicy) -> anyhow::Result<()> {
        self.get_managed_process(pid)?;

        match policy {
            SchedPolicy::OTHER { .. } => (),
            SchedPolicy::FIFO(_) | SchedPolicy::RR(_) => {
                let cgroup = get_pid_cgroup(pid)?;

                if !cgroups.is_managed_cgroup(&cgroup) {
                    anyhow::bail!("Processes can be set to SCHED_FIFO/SCHED_RR only if they are in a managed cgroup");
                }
            },
            _ => anyhow::bail!("unexpected"),
        }

        set_sched_policy(pid, policy)?;

        Ok(())
    }

    fn get_managed_process(&mut self, pid: Pid) -> anyhow::Result<&mut ProcData> {
        if !self.procs.contains_key(&pid) {
            let cgroup = get_pid_cgroup(pid)?;
            self.procs.insert(pid, ProcData::new(cgroup));
        }

        Ok(self.procs.get_mut(&pid).unwrap())
    }
}

impl Default for ProcManager {
    fn default() -> Self {
        Self {
            procs: HashMap::with_capacity(0),
            keep_on_exit: false,
        }
    }
}

impl Drop for ProcManager {
    fn drop(&mut self) {
        if !self.keep_on_exit {
            return;
        }

        for (&pid, data) in self.procs.iter() {
            if let Err(err) = set_sched_policy(pid, SchedPolicy::other()) {
                error!("Couldn't set PID {pid} scheduling policy to SCHED_OTHER: {err}");
            }

            if let Err(err) = assign_pid_to_cgroup(&data.original_cgroup, pid) {
                error!("Couldn't move PID {pid} to its original cgroup: {err}");
            }
        }

        self.procs.clear();
    }
}

impl ProcData {
    pub fn new(original_cgroup: String) -> Self {
        Self { original_cgroup }
    }
}