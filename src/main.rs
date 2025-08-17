use clap::Parser;

use crate::{
    cli::{Cli, Commands},
    config::{Key, deep_keys},
    config_walker::ConfigWalker,
    current_env::CurrentEnv,
};

mod cli;
mod config;
mod config_walker;
mod current_env;
mod shell;

fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();
    let (env, shell) = match cli.command {
        Commands::Get => {
            println!("{}", CurrentEnv::name());
            return Ok(());
        }
        Commands::List => {
            let config = cli::load_config_file(&cli.file)?;
            eprintln!("Available environments:");
            for env in deep_keys(&config) {
                eprintln!("\t{env}");
            }
            return Ok(());
        }
        Commands::Set { env, shell } => (env, shell),
    };
    let config = cli::load_config_file(&cli.file)?;

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
