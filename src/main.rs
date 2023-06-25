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
use futures::future::Either;
use ratatui::backend::CrosstermBackend;
use tokio::pin;
use tokio_util::sync::CancellationToken;
use tonari_actor::{Addr, System, SystemHandle};

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
    let cancel_token = CancellationToken::new();
    let mut system = System::new("default");

    let profiler_addr = Addr::default();
    let tui_addr = system.spawn(Tui::new(&config, profiler_addr.clone())?)?;
    tui_addr.send(TuiMessage::Render)?;

    let python_code = config.python_code.clone();
    let profiler_addr = {
        let tui_addr = tui_addr.clone();
        system
            .prepare_fn(|| {
                let result = Profiler::new(python_code, tui_addr);
                result.unwrap()
            })
            .with_addr(profiler_addr)
            .spawn()
            .unwrap()
    };

    profiler_addr.send(ProfilerMessage::Attach(ProfileTarget {
        pid: config.pid,
        python_bin: config.python_bin,
    }))?;

    let helper_thread = {
        let system: SystemHandle = system.clone();
        let cancel_token = cancel_token.clone();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_time()
                .build()
                .unwrap();

            // task1 and task2 gracefully exit on cancel.
            // task1 and task2 cancel when they fail.
            rt.block_on(async {
                // task 1
                {
                    let cancel_token = cancel_token.clone();
                    tokio::spawn(async move {
                        let cancelled = cancel_token.cancelled();
                        let fut = raw_input::drain_term_input(tui_addr);
                        pin!(cancelled);
                        pin!(fut);
                        match futures::future::select(cancelled, fut).await {
                            Either::Left(_) => {} // cancelled
                            Either::Right((result, _)) => {
                                if let Err(err) = result {
                                    log::error!("Error in raw_input thread: {}", err);
                                }
                                cancel_token.cancel();
                            }
                        }
                    });
                }

                // task 2
                {
                    let cancel_token = cancel_token.clone();
                    tokio::spawn(async move {
                        let cancelled = cancel_token.cancelled();
                        let fut = async {
                            loop {
                                tokio::time::sleep(Duration::from_secs(1)).await;
                                if let Err(err) = profiler_addr.send(ProfilerMessage::Observe) {
                                    match err.reason {
                                        tonari_actor::SendErrorReason::Full => {}
                                        tonari_actor::SendErrorReason::Disconnected => {
                                            log::error!("Profiler actor is disconnected");
                                            break;
                                        }
                                    }
                                }
                            }
                        };
                        tokio::pin!(cancelled);
                        tokio::pin!(fut);
                        match futures::future::select(cancelled, fut).await {
                            Either::Left(_) => {}
                            Either::Right(_) => {
                                cancel_token.cancel();
                            }
                        }
                    });
                }

                cancel_token.cancelled().await;
                log::debug!("tokio runtime finished.");
            });
            system.shutdown().unwrap();
        })
    };

    // system fails first -> cancel -> join
    // thread fails first -> cancel -> shutdown
    system.run()?;
    cancel_token.cancel();
    helper_thread.join().unwrap();

    Ok(())
}
