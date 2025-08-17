use std::{
    fs,
    path::{Path, PathBuf},
};

use clap::{Args, Parser, Subcommand};
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
}

#[derive(Debug, Clone, Args)]
pub struct List {
    #[arg(short, long, help = "hello")]
    pub file: Option<PathBuf>,
}

#[derive(Debug, Clone, Args)]
pub struct Set {
    #[arg(short, long)]
    pub file: Option<PathBuf>,
    /// The name of the environment to select; leave blank to only set global
    /// options.
    #[arg(default_value = "")]
    pub env: String,

    #[arg(short, long)]
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
