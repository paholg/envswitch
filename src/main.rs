use std::{env, fs, path::PathBuf};

use ahash::AHashMap;
use clap::{Parser, Subcommand};
use eyre::eyre;
use itertools::Itertools;

use crate::{
    config::{Key, Table},
    shell::Shell,
};

mod config;
mod shell;

pub const ENVSWITCH_VAR: &str = "ENVSWITCH_ENV";

pub struct CurrentEnv {
    env: Option<String>,
    vars: Vec<String>,
}

impl CurrentEnv {
    fn new() -> eyre::Result<Self> {
        match env::var(ENVSWITCH_VAR) {
            Ok(value) => {
                let Some((env, vars)) = value.split_once(':') else {
                    todo!()
                };

                Ok(Self {
                    env: Some(env.to_string()),
                    vars: vars.split(',').map(ToString::to_string).collect(),
                })
            }
            Err(_) => Ok(Self {
                env: None,
                vars: Vec::new(),
            }),
        }
    }

    fn clear_commands(&self, shell: &Shell) -> impl Iterator<Item = String> {
        self.vars.iter().map(|var| shell.clear_var(var))
    }

    fn name(&self) -> &str {
        self.env.as_deref().unwrap_or_default()
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

#[derive(Parser, Debug)]
#[command(version, about, long_about)]
struct Args {
    #[arg(short, long)]
    shell: Shell,

    #[command(subcommand)]
    action: Action,
}

#[derive(Clone, Debug, Subcommand)]
enum Action {
    Get,
    Set {
        env: String,

        #[arg(short, long, default_value = "./envswitch.toml")]
        file: PathBuf,
    },
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    let current_env = CurrentEnv::new()?;
    let (env, file) = match args.action {
        Action::Get => {
            println!("{}", current_env.name());
            return Ok(());
        }
        Action::Set { env, file } => (env, file),
    };

    let shell = args.shell;

    let file = fs::read(&file)?;
    let config: Table = toml::from_slice(&file)?;

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

    Ok(())
}

#[derive(Debug, Default)]
struct Walker<'a> {
    vals: AHashMap<&'a str, &'a str>,
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
        let inner = config
            .get(head)
            .ok_or_else(|| eyre!("missing key '{head}'"))?
            .as_table()
            .ok_or_else(|| eyre!("key '{head}' does not correspond to a table"))?;

        self.walk(inner, keys)?;

        Ok(())
    }
}
