use clap::Parser;
use clap_verbosity_flag::Verbosity;
use color_eyre::{eyre::ContextCompat, Result};
use std::path::Path;

use crate::error::Error;

#[derive(Parser)]
#[command(version)]
pub struct Args {
    // target
    #[arg()]
    pub python_code: String,
    #[arg(long, default_value_t=-1)]
    pub pid: i32,
    /// relative path (./python) or symlink (/proc/148213/exe) is accepted
    #[arg(long)]
    pub python_bin: Option<String>,
    #[command(flatten)]
    pub verbose: Verbosity<clap_verbosity_flag::InfoLevel>,
}

pub struct Config {
    pub pid: i32,
    pub python_bin: String,
    pub python_code: String, // code path
}

impl Config {
    pub fn new(args: Args) -> Result<Self> {
        Ok(Self {
            pid: args.pid,
            python_bin: determine_python(&args),
            python_code: Path::new(&args.python_code)
                .canonicalize()
                .map_err(|source| Error::CannotReadFile {
                    source,
                    path: args.python_code.clone(),
                })?
                .to_str()
                .context("python code path is not UTF-8")?
                .to_string(),
        })
    }
}

fn determine_python(args: &Args) -> String {
    // pid python_bin
    //  -1     x           python
    //  -1     o           (args.python_bin)
    //   o     x           /proc/(pid)/exe
    //   o     o           (args.python_bin)
    match (args.pid, &args.python_bin) {
        (-1, None) => "python".to_string(),
        (-1, Some(path)) => path.clone(),
        (pid, None) => format!("/proc/{pid}/exe"),
        (_, Some(path)) => {
            log::warn!("You don't have to specify both --pid and --python-bin");
            path.clone()
        }
    }
}
