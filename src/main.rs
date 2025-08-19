use std::borrow::Cow;

use clap::Parser;
use color_eyre::config::HookBuilder;

use crate::{
    cli::{Cli, Commands, Complete, Set, Setup},
    config::{Key, deep_keys},
    config_walker::ConfigWalker,
    current_env::CurrentEnv,
};

mod cli;
mod config;
mod config_walker;
mod current_env;
mod shell;

#[cfg(test)]
mod test;

fn get() -> eyre::Result<()> {
    println!("{}", CurrentEnv::name());
    Ok(())
}

fn set(args: Set) -> eyre::Result<()> {
    let Set {
        config,
        env,
        shell,
        list,
    } = args;
    let config = cli::load_config_file(config.file.as_deref())?;
    if list {
        eprintln!("Available environments:");
        for env in deep_keys(&config) {
            eprintln!("  {env}");
        }
        return Ok(());
    }

    let current_env = CurrentEnv::new()?;

    let keys = env
        .split('.')
        .map(|k| Key::try_from(k.to_string()))
        .collect::<eyre::Result<Vec<_>>>()?;

    let walker = ConfigWalker::new(&config, keys.iter())?;

    let commands = current_env
        .clear_commands(&shell)
        .chain([current_env.set(&shell, &env, walker.vals.keys().copied())])
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

fn complete(args: Complete) -> eyre::Result<()> {
    let config = cli::load_config_file(args.config.file.as_deref())?;

    if args.env.len() > 1 {
        // Already complete
        return Ok(());
    }

    let env = args.env.first().map(Cow::Borrowed).unwrap_or_default();
    let env = env.as_str();

    // We could do this more efficiently by filtering as we construct deep_keys, but it shouldn't
    // matter in practice.
    for key in deep_keys(&config).filter(|key| key.starts_with(env)) {
        println!("{key}");
    }

    Ok(())
}

fn setup(args: Setup) -> eyre::Result<()> {
    println!("{}", args.shell.setup());
    Ok(())
}

fn main() -> eyre::Result<()> {
    HookBuilder::default()
        .display_env_section(false)
        .install()?;

    let cli = Cli::parse();

    match cli.command {
        Commands::Get => get(),
        Commands::Set(args) => set(args),
        Commands::Setup(args) => setup(args),
        Commands::Complete(args) => complete(args),
    }
}
