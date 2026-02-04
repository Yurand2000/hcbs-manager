use hcbs_manager::prelude::*;

#[derive(Debug, clap::Parser)]
struct Args {
    /// Max bandwidth of the Cgroup hierarchy
    #[arg(short='b', long="bandwidth", default_value="0.9")]
    runtime_bw: f64,

    /// Reset changes on exit
    ///
    /// This resets the cgroup hierarchy allocated bandwidth and changes all
    /// touched processes back to SCHED_OTHER, moving them to the cgroup they
    /// were into before being moved around by commands.
    #[arg(short='e')]
    reset_on_exit: bool,
}

struct ResetData {
    old_runtime_us: u64,
}

fn main() -> anyhow::Result<()> {
    let args = clap::Parser::parse();

    // Debug Logging
    env_logger::init();

    // Setup HCBS
    let reset_data = setup_hcbs(&args)?;

    // Run Manager
    let res = Controller::new(
            args.reset_on_exit
        ).mount();

    // Reset HCBS
    reset_hcbs(&args, reset_data)?;

    res
}

fn setup_hcbs(args: &Args) -> anyhow::Result<ResetData> {
    use hcbs_utils::prelude::*;

    // Mount Cgroup filesystem and CPU controller
    mount_cgroup_fs()?;

    // Reserve bandwidth for the CGroup hierarchy
    let period_us = get_cgroup_period_us(ROOT_CGROUP)?;
    let old_runtime_us = get_cgroup_runtime_us(ROOT_CGROUP)?;
    let runtime_us = (args.runtime_bw * period_us as f64).floor() as u64;
    set_cgroup_runtime_us(ROOT_CGROUP, runtime_us)?;

    Ok(ResetData {
        old_runtime_us
    })
}

fn reset_hcbs(args: &Args, data: ResetData) -> anyhow::Result<()> {
    use hcbs_utils::prelude::*;

    if !args.reset_on_exit {
        return Ok(());
    }

    set_cgroup_runtime_us(ROOT_CGROUP, data.old_runtime_us)?;

    Ok(())
}