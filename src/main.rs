use clap::Parser;

use crate::{
    cli::{Cli, Commands, List, Set},
    config::{Key, deep_keys},
    config_walker::ConfigWalker,
    current_env::CurrentEnv,
};

mod cli;
mod config;
mod config_walker;
mod current_env;
mod shell;

fn list(args: List) -> eyre::Result<()> {
    let config = cli::load_config_file(args.file.as_deref())?;
    eprintln!("Available environments:");
    for env in deep_keys(&config) {
        eprintln!("\t{env}");
    }
    Ok(())
}

fn get() -> eyre::Result<()> {
    println!("{}", CurrentEnv::name());
    Ok(())
}

fn set(args: Set) -> eyre::Result<()> {
    let Set { file, env, shell } = args;
    let config = cli::load_config_file(file.as_deref())?;

    let current_env = CurrentEnv::new()?;

    let keys = env
        .split('.')
        .map(|k| Key::try_from(k.to_string()))
        .collect::<eyre::Result<Vec<_>>>()?;

    let walker = ConfigWalker::new(&config, keys.iter())?;

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

fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();

    match cli.command {
        Commands::Get => get(),
        Commands::List(args) => list(args),
        Commands::Set(args) => set(args),
    }
}
