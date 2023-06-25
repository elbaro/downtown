pub mod cli;
pub mod error;
pub mod profiler;
pub mod tui;

use std::time::Duration;

use crate::{
    cli::{Args, Config},
    profiler::{types::ProfileTarget, Profiler, ProfilerMessage},
    tui::{raw_input, Tui, TuiMessage},
};
use clap::Parser;

use color_eyre::Result;
use ratatui::backend::CrosstermBackend;
use tonari_actor::{Addr, System};

pub type Terminal = ratatui::Terminal<CrosstermBackend<std::io::Stdout>>;

fn main() -> Result<()> {
    color_eyre::install()?;
    dotenvy::dotenv()?;
    let args = Args::parse();
    pretty_env_logger::formatted_builder()
        .filter_level(args.verbose.log_level_filter())
        .init();

    sudo::with_env(&[])
        .map_err(|_| color_eyre::eyre::eyre!("The program needs to run in privileged mode"))?;

    let config = Config::new(args)?;
    validate_config(&config)?;
    run_system(config)?;
    Ok(())
}

fn validate_config(_config: &Config) -> Result<()> {
    // config.pid
    // config.python_code
    // config.python_bin
    Ok(())
}

fn run_system(config: Config) -> Result<()> {
    let mut system = System::new("default");

    let profiler_addr = Addr::default();
    let tui_addr = system.spawn(Tui::new(&config, profiler_addr.clone())?)?;
    tui_addr.send(TuiMessage::Render)?;
    {
        let system = system.clone();
        let tui_addr = tui_addr.clone();
        std::thread::spawn(move || raw_input::drain(system, tui_addr).unwrap())
    };

    let python_code = config.python_code.clone();
    let profiler_addr = system
        .prepare_fn(|| Profiler::new(python_code, tui_addr).unwrap())
        .with_addr(profiler_addr)
        .spawn()?;

    profiler_addr.send(ProfilerMessage::Attach(ProfileTarget {
        pid: config.pid,
        python_bin: config.python_bin,
    }))?;

    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(2));
        profiler_addr.send(ProfilerMessage::Observe).unwrap();
    });

    system.run()?;
    Ok(())
}
