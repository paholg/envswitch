use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

use eyre::{Context, eyre};
use indexmap::IndexMap;

use crate::shell::Shell;

pub struct ScriptResult {
    base_env: String,
    output: Output,
}

fn parse_env(env: &str) -> IndexMap<&str, &str> {
    env.lines()
        .filter_map(|line| line.split_once('='))
        .collect()
}

impl ScriptResult {
    pub fn status(&self) -> i32 {
        self.output.status.code().unwrap()
    }

    pub fn env_diff(&self) -> IndexMap<&str, &str> {
        let base_env = parse_env(&self.base_env);
        let stdout: &str = str::from_utf8(&self.output.stdout).unwrap();

        let mut env = parse_env(&stdout);
        for (key, val) in base_env {
            if env.get(&key) == Some(&val) {
                env.swap_remove(&key);
            }
        }

        env
    }

    pub fn assert_stderr_includes(&self, s: &str) {
        let stderr = str::from_utf8(&self.output.stderr).unwrap();

        assert!(
            stderr.contains(s),
            "stderr: '{stderr}' does not contain '{s}'"
        );
    }
}

fn get_env(dir: &Path, shell: &Shell, name: &str, command: &str) -> eyre::Result<Output> {
    Command::new(shell.to_string())
        .output()
        .wrap_err_with(|| eyre!("Could not run {shell}"))?;

    let dir_display = dir.display();
    let bin = escargot::CargoBuild::new()
        .bin("envswitch")
        .current_release()
        .current_target()
        .run()?;
    let bin = bin.path();
    let prefix = shell.script_prefix(bin);
    let script_body = &[
        prefix,
        format!("cd {dir_display}"),
        command.to_string(),
        "env".to_string(),
    ]
    .join("\n");
    println!("SCRIPT:\n{script_body}");

    let script_path = dir.join(name);
    fs::write(&script_path, script_body.as_bytes())?;
    let output = Command::new(shell.to_string()).arg(&script_path).output()?;
    Ok(output)
}

fn run_command_inner(
    config: &toml::Table,
    shell: Shell,
    dir: &Path,
    command: &str,
) -> eyre::Result<ScriptResult> {
    let config_path = dir.join("envswitch.toml");
    fs::write(&config_path, toml::to_string(&config)?.as_bytes())?;

    // We first run without the command to get a baseline for the ENV.
    let base_env = get_env(dir, &shell, "base_env", "")?.stdout.try_into()?;
    let output = get_env(dir, &shell, "test_script", &shell.try_cmd(command))?;

    Ok(ScriptResult { base_env, output })
}

pub fn run_command(shell: Shell, config: &toml::Table, command: &str) -> ScriptResult {
    let _ = color_eyre::install();

    let dir = tempfile::tempdir().unwrap().keep();
    let result = run_command_inner(config, shell, &dir, command);
    // We opted out of automatic cleanup, so we need to be sure to clean up after ourselves.
    std::fs::remove_dir_all(dir).unwrap();

    let result = result.unwrap();
    println!("{:?}", result.output);
    result
}
