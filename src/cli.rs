use std::{
    fs,
    path::{Path, PathBuf},
};

use clap::{Parser, Subcommand};

use crate::{config::Table, shell::Shell};

const ABOUT: &str = "A simple tool for managing sets of environment variables

Run with no arguments to see the current environment setting.";

#[derive(Parser, Debug)]
#[command(version, about = ABOUT)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(short, long, default_value = "./envswitch.toml")]
    pub file: PathBuf,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Show the name of the current environment
    Get,
    /// List available environments
    List,
    /// Set the environment
    Set {
        /// The name of the environment to select; leave blank to only set global
        /// options.
        #[arg(default_value = "")]
        env: String,

        #[arg(short, long)]
        shell: Shell,
    },
}

pub fn load_config_file(path: &Path) -> eyre::Result<Table> {
    let file = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(_) => Vec::new(),
    };
    let config: Table = toml::from_slice(&file)?;

    Ok(config)
}
