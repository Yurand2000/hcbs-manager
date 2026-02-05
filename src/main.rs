use hcbs_manager::prelude::*;
use hcbs_utils::prelude::*;

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

    /// Log level
    ///
    /// Available values: "off", "error", "warn", "info", "debug", "trace"
    #[arg(long="log-level", default_value="warn")]
    log_level: log::LevelFilter,
}

fn main() -> anyhow::Result<()> {
    let args: Args = clap::Parser::parse();

    // Debug Logging
    env_logger::builder()
        .filter_level(args.log_level)
        .init();

    // Set manager to run on real-time scheduling policy
    assign_pid_to_cgroup(ROOT_CGROUP, 0)?;
    set_sched_policy(0, SchedPolicy::FIFO(99))?;

    // Setup System for Real-Time workloads
    setup_reset_helper(
        setup_realtime_system,
        reset_realtime_system,
        // Setup HCBS Hierarchy
        || setup_reset_helper(
            || setup_hcbs(&args),
            |data| reset_hcbs(&args, data),
            // Start HCBS Manager
            || {
                Controller::new(
                    args.reset_on_exit
                ).mount()
            }
        )
    )
}

struct HCBSResetData {
    old_runtime_us: u64,
}

fn setup_hcbs(args: &Args) -> anyhow::Result<HCBSResetData> {
    // Mount Cgroup filesystem and CPU controller
    mount_cgroup_fs()?;

    // Reserve bandwidth for the CGroup hierarchy
    let period_us = get_cgroup_period_us(ROOT_CGROUP)?;
    let old_runtime_us = get_cgroup_runtime_us(ROOT_CGROUP)?;
    let runtime_us = (args.runtime_bw * period_us as f64).floor() as u64;
    set_cgroup_runtime_us(ROOT_CGROUP, runtime_us)?;

    Ok(HCBSResetData {
        old_runtime_us,
    })
}

fn reset_hcbs(args: &Args, data: HCBSResetData) -> anyhow::Result<()> {
    if !args.reset_on_exit {
        return Ok(());
    }

    // Disable Cgroup hierarchy
    set_cgroup_runtime_us(ROOT_CGROUP, data.old_runtime_us)?;

    Ok(())
}

struct RealtimeResetData {
    hyperthreading_enabled: bool,
    intel_pstate: intel::PState,
    cpu_freq_governor_data: Vec<(CpuID, CpuFrequencyGovernorData)>,
    cpu_idle_states: Vec<(CpuID, CpuIdleStates)>,
}

fn setup_realtime_system() -> anyhow::Result<RealtimeResetData> {
    // Hyperthreading and Frequency Governors
    if !intel::has_intel_pstate()? {
        anyhow::bail!("Only Intel CPUs supported")
    }

    // Get current system state
    let cpus = CpuSet::all()?;
    let hyperthreading_enabled = hyperthreading_enabled()?;
    let intel_pstate = intel::get_pstate()?;
    let cpu_freq_governor_data =
        cpus.iter()
        .map(|&cpu| Ok((cpu, get_cpu_frequency_governor(cpu)?)) )
        .collect::<anyhow::Result<_>>()?;
    let cpu_idle_states =
        cpus.iter()
        .map(|&cpu| Ok((cpu, get_cpu_idle_state(cpu)?)) )
        .collect::<anyhow::Result<_>>()?;

    // Set Real-Time system state
    if hyperthreading_enabled {
        disable_hyperthreading()?;
    }
        // refresh CPUs as disabling hyperthreading switches off some logical cores.
    let cpus = CpuSet::all()?;
    intel::set_pstate(intel::PState::fix_performance())?;
    for &cpu in cpus.iter() {
        let freqs = get_cpu_frequency(cpu)?;

        set_cpu_frequency_governor(cpu, CpuFrequencyGovernorData::fixed_frequency(freqs.max_frequency_mhz))?;
    }
    for &cpu in cpus.iter() {
        let cpu = cpu as u32;
        set_cpu_idle_state(cpu, CpuIdleStates::disabled_for_cpu(cpu)?)?;
    }

    Ok(RealtimeResetData {
        hyperthreading_enabled,
        intel_pstate,
        cpu_freq_governor_data,
        cpu_idle_states
    })
}

fn reset_realtime_system(data: RealtimeResetData) -> anyhow::Result<()> {
    // Reset Real-Time system state
    if data.hyperthreading_enabled { enable_hyperthreading()? }
    intel::set_pstate(data.intel_pstate)?;
    for (cpu, data) in data.cpu_freq_governor_data.into_iter() {
        set_cpu_frequency_governor(cpu, data)?;
    }
    for (cpu, data) in data.cpu_idle_states.into_iter() {
        set_cpu_idle_state(cpu, data)?;
    }

    Ok(())
}

fn setup_reset_helper<F, D, FSetup, FReset>(
    setup: FSetup,
    reset: FReset,
    f: F,
) -> anyhow::Result<()>
    where
        F: FnOnce() -> anyhow::Result<()>,
        FSetup: FnOnce() -> anyhow::Result<D>,
        FReset: FnOnce(D) -> anyhow::Result<()>,
{
    let reset_data = setup()?;

    let result = f();

    reset(reset_data)?;

    result
}