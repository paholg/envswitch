use std::env;

use eyre::eyre;
use itertools::Itertools;

use crate::shell::Shell;

pub const ENVSWITCH_VAR: &str = "ENVSWITCH_ENV";

pub struct CurrentEnv {
    vars: Vec<String>,
}

impl CurrentEnv {
    pub fn name() -> String {
        match env::var(ENVSWITCH_VAR) {
            Ok(value) => value
                .split_once(':')
                .map(|(env, _)| env.to_string())
                .unwrap_or_default(),
            Err(_) => String::new(),
        }
    }

    pub fn new() -> eyre::Result<Self> {
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

    pub fn clear_commands(&self, shell: &Shell) -> impl Iterator<Item = String> {
        self.vars
            .iter()
            .filter(|var| !var.is_empty())
            .map(|var| shell.clear_var(var))
    }

    pub fn set<'a>(
        &self,
        shell: &Shell,
        env: &'a str,
        vars: impl Iterator<Item = &'a str>,
    ) -> String {
        let mut value = String::new();
        value.push_str(env);
        value.push(':');

        for s in Itertools::intersperse(vars, ",") {
            value.push_str(s);
        }

        shell.set_var(ENVSWITCH_VAR, &value)
    }
}
