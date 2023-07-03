#![allow(clippy::manual_non_exhaustive)]

pub mod cli;
pub mod error;
pub mod profiler;
pub mod tui;

use crate::{
    cli::{Args, Config},
    profiler::{types::ProfileTarget, Profiler, ProfilerCommand},
    tui::{raw_input::term_input_stream, Tui, TuiMessage},
};
use clap::Parser;
use color_eyre::Result;
use futures::StreamExt;
use ratatui::backend::CrosstermBackend;
use std::time::Duration;
use tokio_stream::wrappers::IntervalStream;
use xtra::Mailbox;

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

#[tokio::main(flavor = "current_thread")]
async fn run_system(config: Config) -> Result<()> {
    let code = tui::code::PythonCode::new(config.python_code.clone()).await?;

    let profiler_addr = xtra::spawn_tokio(
        Profiler::new(config.python_code.clone())?,
        Mailbox::unbounded(),
    );
    let tui_addr = xtra::spawn_tokio(
        Tui::new(&config, code, profiler_addr.clone())?,
        Mailbox::unbounded(),
    );
    let mut handles = vec![];

    let tui_addr_ = tui_addr.clone();
    handles.push(tokio::spawn(xtra::scoped(
        &tui_addr,
        term_input_stream().forward(tui_addr_.into_sink()),
    )));

    let tui_addr_ = tui_addr.clone();
    handles.push(tokio::spawn(xtra::scoped(
        &tui_addr,
        IntervalStream::new(tokio::time::interval(Duration::from_secs(1)))
            .map(|_| Ok(TuiMessage::Update))
            .forward(tui_addr_.into_sink()),
    )));

    tui_addr.send(TuiMessage::Render).await?;
    profiler_addr
        .send(ProfilerCommand::Attach(ProfileTarget {
            pid: config.pid,
            python_bin: config.python_bin,
        }))
        .await??;

    for handle in handles {
        if let Some(result) = handle.await? {
            result?;
        }
    }

    Ok(())
}
