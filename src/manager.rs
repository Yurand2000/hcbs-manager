use std::collections::HashSet;

use hcbs_utils::prelude::*;

#[derive(Debug)]
pub struct CgroupManager {
    cgroups: HashSet<String>,
}

pub struct Reservation {
    runtime_us: u64,
    period_us: u64,
}

impl CgroupManager {
    const MAX_RESOURCE: f64 = 0.95;

    pub fn new() -> Self {
        Self {
            cgroups: HashSet::new(),
        }
    }

    pub fn create_cgroup(&mut self, name: &str, request: Reservation) -> anyhow::Result<()> {
        if self.cgroups.contains(name) {
            return Err(anyhow::format_err!("Cgroup {} already exists.", cgroup_abs_path(name)));
        }

        if !self.run_admission_test(&request)? {
            return Err(anyhow::format_err!("Cgroup {} cannot be allocated: insufficient resources.", cgroup_abs_path(name)));
        }

        Cgroup::create(name, request)
            .map_err(|err| anyhow::format_err!("Cgroup {} cannot be allocated: {err}", cgroup_abs_path(name)))?;

        self.cgroups.insert(name.to_owned());

        Ok(())
    }

    pub fn destroy_cgroup(&mut self, name: &str) -> anyhow::Result<()> {
        self.cgroups.get(name)
            .ok_or_else(|| anyhow::format_err!("Cgroup {} does not exist.", cgroup_abs_path(name)))
            .and_then(|name| {
                Cgroup::destroy(name)
                .map_err(|err| anyhow::format_err!("Cgroup {} cannot be destroyed: {err}", cgroup_abs_path(name)))
            })?;

        self.cgroups.remove(name);

        Ok(())
    }

    fn run_admission_test(&self, request: &Reservation) -> anyhow::Result<bool> {
        let current_allocation =
            self.cgroups.iter()
            .map(|name| -> anyhow::Result<_> {
                let runtime_us = get_cgroup_runtime_us(name)?;
                let period_us = get_cgroup_period_us(name)?;

                Ok(runtime_us as f64 / period_us as f64)
            })
            .try_fold(0.0, |acc, util| -> anyhow::Result<_> { Ok(acc + util?) })?;

        let new_allocation =
            request.runtime_us as f64 / request.period_us as f64;

        Ok(new_allocation + current_allocation <= Self::MAX_RESOURCE)
    }
}

impl Drop for CgroupManager {
    fn drop(&mut self) {
        for name in self.cgroups.iter() {
            Cgroup::force_destroy(name).unwrap()
        };
    }
}

struct Cgroup;

impl Cgroup {
    pub fn create(name: &str, reservation: Reservation) -> anyhow::Result<()> {
        create_cgroup(name)?;

        set_cgroup_period_us(name, reservation.period_us)?;
        set_cgroup_runtime_us(name, reservation.runtime_us)?;
        Ok(())
    }

    pub fn destroy(name: &str) -> anyhow::Result<()> {
        set_cgroup_runtime_us(name, 0)?;

        delete_cgroup(name)?;

        Ok(())
    }

    fn force_destroy(name: &str) -> anyhow::Result<()> {
        for pid in cgroup_pids(name)? {
            kill_pid(pid);
        }

        std::thread::sleep(std::time::Duration::from_millis(100));

        Self::destroy(name)
    }
}