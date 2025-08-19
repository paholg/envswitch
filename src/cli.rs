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
    /// Set the environment
    Set(Set),
    /// Generate a command to integrate envswitch with your shell
    Setup(Setup),
    #[clap(hide = true)]
    Complete(Complete),
}

#[derive(Debug, Clone, Args)]
// Clap makes it too hard to have help printed to stderr, so we disable it
// entirely here to prevent it from being `source`d by our shell function.
#[command(disable_help_flag = true)]
pub struct Set {
    #[command(flatten)]
    pub config: ConfigPath,
    /// The name of the environment to select; leave blank to only set global
    /// options.
    #[arg(default_value = "", value_hint = ValueHint::Other)]
    pub env: String,

    #[arg(short, long)]
    pub shell: Shell,

    /// List available environments instead of setting any.
    #[arg(short, long)]
    pub list: bool,
}

#[derive(Debug, Clone, Args)]
pub struct ConfigPath {
    #[arg(short, long, value_hint = ValueHint::FilePath, help = "path to config file [defaults: ./envswitch.toml]")]
    pub file: Option<PathBuf>,
}

#[derive(Debug, Clone, Args)]
pub struct Complete {
    #[command(flatten)]
    pub config: ConfigPath,

    // NOTE: We accept multiple positional arguments here just to make completion
    // not error if you hit TAB too many times. We only use the first one.
    #[arg(default_value = "")]
    pub env: Vec<String>,

    // We don't use this, but we need to respect all arguments that might be
    // passed into `es`.
    #[arg(short, long)]
    list: bool,
}

#[derive(Debug, Clone, Args)]
pub struct Setup {
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
