use std::{env, fs, ops::Deref, path::PathBuf};

use clap::Parser;
use eyre::eyre;
use indexmap::IndexMap;
use itertools::Itertools;

use crate::{
    config::{Key, Table, deep_keys},
    shell::Shell,
};

mod config;
mod shell;

pub const ENVSWITCH_VAR: &str = "ENVSWITCH_ENV";

pub struct CurrentEnv {
    vars: Vec<String>,
}

impl CurrentEnv {
    fn name() -> String {
        match env::var(ENVSWITCH_VAR) {
            Ok(value) => value
                .split_once(':')
                .map(|(env, _)| env.to_string())
                .unwrap_or_default(),
            Err(_) => String::new(),
        }
    }

    fn new() -> eyre::Result<Self> {
        match env::var(ENVSWITCH_VAR) {
            Ok(value) => {
                let Some((_env_name, vars)) = value.split_once(':') else {
                    return Err(eyre!(
                        "Invalid {ENVSWITCH_VAR} variable; please inspect and clear it"
                    ));
                };

                Ok(Self {
                    vars: vars.split(',').map(ToString::to_string).collect(),
                })
            }
            Err(_) => Ok(Self { vars: Vec::new() }),
        }
    }

    fn clear_commands(&self, shell: &Shell) -> impl Iterator<Item = String> {
        self.vars
            .iter()
            .filter(|var| !var.is_empty())
            .map(|var| shell.clear_var(var))
    }

    fn set<'a>(&self, shell: &Shell, env: &'a str, vars: impl Iterator<Item = &'a str>) -> String {
        let mut value = String::new();
        value.push_str(env);
        value.push(':');

        for s in Itertools::intersperse(vars, ",") {
            value.push_str(s);
        }

        shell.set_var(ENVSWITCH_VAR, &value)
    }
}

const ABOUT: &str = "A simple tool for managing sets of environment variables

Run with no arguments to see the current environment setting.";

#[derive(Parser, Debug)]
#[command(version, about = ABOUT)]
struct Args {
    /// The name of the environment to select; leave blank to only set global
    /// options.
    #[arg(default_value = "")]
    env: String,

    /// List currently available environments and exit.
    #[arg(short, long)]
    list: bool,

    #[arg(short, long)]
    shell: Shell,

    #[arg(short, long, default_value = "./envswitch.toml")]
    file: PathBuf,
}

fn main() -> eyre::Result<()> {
    // I can find no good way with clap to support either a set of args or no
    // args without hiding them behind subcommands, so we just skip clap when
    // there are no args.
    if env::args().len() == 1 {
        println!("{}", CurrentEnv::name());
        return Ok(());
    }

    color_eyre::install()?;
    let current_env = CurrentEnv::new()?;

    let Args {
        env,
        shell,
        file,
        list,
    } = Args::parse();

    let file = match fs::read(&file) {
        Ok(bytes) => bytes,
        Err(_) => {
            if !list {
                eprintln!("No file found; clearing environment");
            }
            Vec::new()
        }
    };
    let config: Table = toml::from_slice(&file)?;

    if list {
        eprintln!("Available environments:");
        for env in deep_keys(&config) {
            eprintln!("\t{env}");
        }
        return Ok(());
    }

    let keys = env
        .split('.')
        .map(|k| Key::try_from(k.to_string()))
        .collect::<eyre::Result<Vec<_>>>()?;

    let walker = Walker::new(&config, keys.iter())?;

    let commands = current_env
        .clear_commands(&shell)
        .chain([current_env.set(&shell, &env, walker.vals.keys().map(|k| *k))])
        .chain(walker.set_commands(&shell));

    for command in commands {
        println!("{command}");
    }

    let variables = walker.variables();

    if env.is_empty() && variables.is_empty() {
        eprintln!("Environment cleared");
    } else {
        eprintln!("Environment set: {env} {variables}");
    }

    Ok(())
}

#[derive(Debug, Default)]
struct Walker<'a> {
    vals: IndexMap<&'a str, &'a str>,
}

impl<'a> Walker<'a> {
    fn new(config: &'a Table, keys: impl Iterator<Item = &'a Key>) -> eyre::Result<Self> {
        let mut this = Self::default();
        this.walk(config, keys)?;
        Ok(this)
    }

    fn set_commands(&self, shell: &Shell) -> impl Iterator<Item = String> {
        self.vals
            .iter()
            .map(|(var, value)| shell.set_var(var, value))
    }

    fn variables(&self) -> String {
        Itertools::intersperse(self.vals.keys().map(Deref::deref), " ").collect()
    }

    fn walk(
        &mut self,
        config: &'a Table,
        mut keys: impl Iterator<Item = &'a Key>,
    ) -> eyre::Result<()> {
        // First we track any variables that are set at this level:
        let variables = config
            .iter()
            .flat_map(|(k, v)| v.as_string().map(|v| (k, v)));
        for (var, value) in variables {
            self.vals.insert(&var, value);
        }

        // Now we go to the next level:
        let Some(head) = keys.next() else {
            return Ok(());
        };
        if head.is_empty() {
            return Ok(());
        }
        let inner = config
            .get(head)
            .ok_or_else(|| eyre!("missing key '{head}'"))?
            .as_table()
            .ok_or_else(|| eyre!("key '{head}' does not correspond to a table"))?;

        self.walk(inner, keys)?;

        Ok(())
    }
}
