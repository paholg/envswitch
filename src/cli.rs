use std::{
    fs,
    path::{Path, PathBuf},
};

use clap::{Args, Parser, Subcommand, ValueHint};
use eyre::{Context, eyre};

use crate::{config::Table, shell::Shell};

const ABOUT: &str = "A simple tool for managing sets of environment variables

Run with no arguments to see the current environment setting.";

const DEFAULT_FILE: &str = "./envswitch.toml";

#[derive(Parser, Debug)]
#[command(version, about = ABOUT)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Show the name of the current environment
    Get,
    /// List available environments
    List(List),
    /// Set the environment
    Set(Set),
    /// Generate shell completions
    Completions(Completions),
}

#[derive(Debug, Clone, Args)]
pub struct List {
    #[command(flatten)]
    pub config: ConfigPath,
}

#[derive(Debug, Clone, Args)]
pub struct Set {
    #[command(flatten)]
    pub config: ConfigPath,
    /// The name of the environment to select; leave blank to only set global
    /// options.
    #[arg(default_value = "", value_hint = ValueHint::Other)]
    pub env: String,

    #[arg(short, long)]
    pub shell: Shell,
}

#[derive(Debug, Clone, Args)]
pub struct ConfigPath {
    #[arg(short, long, value_hint = ValueHint::FilePath, help = "path to config file [defaults: ./envswitch.toml]")]
    pub file: Option<PathBuf>,
}

// TODO: Try to get nice completions working for `env`.
// fn env_completer(current: &std::ffi::OsStr) -> Vec<CompletionCandidate> {
//     todo!("current: {current:?}")
// }

#[derive(Debug, Clone, Args)]
pub struct Completions {
    pub shell: Shell,
}

pub fn load_config_file(path: Option<&Path>) -> eyre::Result<Table> {
    fn load_file_inner(path: Option<&Path>) -> eyre::Result<Table> {
        let bytes = match path {
            Some(path) => fs::read(path)?,
            None => {
                if fs::exists(DEFAULT_FILE)? {
                    fs::read(DEFAULT_FILE)?
                } else {
                    Vec::new()
                }
            }
        };
        let config: Table = toml::from_slice(&bytes)?;

        Ok(config)
    }

    load_file_inner(path).wrap_err_with(|| match path {
        Some(p) => eyre!("Failed to read file {}", p.display()),
        None => eyre!("Failed to read default file {DEFAULT_FILE}"),
    })
}
