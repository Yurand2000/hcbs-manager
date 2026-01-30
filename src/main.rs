use hcbs_manager::prelude::*;

fn main() -> anyhow::Result<()> {
    // Debug Logging
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    HCBSController::new().mount()?;

    Ok(())
}
