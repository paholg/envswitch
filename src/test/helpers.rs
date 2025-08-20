use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

use eyre::{Context, eyre};
use indexmap::IndexMap;
use rexpect::{reader::Options, spawn_with_options};

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

fn execute_script(dir: &Path, shell: Shell, name: &str, command: &str) -> eyre::Result<Output> {
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
    let script_body = &[prefix, format!("cd {dir_display}"), command.to_string()].join("\n");
    println!("SCRIPT {name}:\n{script_body}");

    let script_path = dir.join(name);
    fs::write(&script_path, script_body.as_bytes())?;
    let output = Command::new(shell.to_string()).arg(&script_path).output()?;
    Ok(output)
}

fn run_test<F: Fn(&Path, Shell) -> eyre::Result<T>, T>(
    shell: Shell,
    config: &toml::Table,
    test: F,
) -> T {
    fn run_inner<F: Fn(&Path, Shell) -> eyre::Result<T>, T>(
        dir: &Path,
        config: &toml::Table,
        shell: Shell,
        test: F,
    ) -> eyre::Result<T> {
        let config_path = dir.join("envswitch.toml");
        fs::write(&config_path, toml::to_string(&config)?.as_bytes())?;

        test(dir, shell)
    }

    let _ = color_eyre::install();
    // We opt out of automatic cleanup, so we need to be sure to clean up after ourselves.
    let dir = tempfile::tempdir().unwrap().keep();
    let result = run_inner(&dir, config, shell, test);
    std::fs::remove_dir_all(dir).unwrap();

    result.unwrap()
}

pub fn run_command(shell: Shell, config: &toml::Table, command: &str) -> ScriptResult {
    run_test(shell, config, |dir, shell| {
        // We first run without the command to get a baseline for the ENV.
        let base_env = execute_script(dir, shell, "base_env", "env")?
            .stdout
            .try_into()?;
        let output = execute_script(
            dir,
            shell,
            "base_env",
            &[&shell.try_cmd(command), "env"].join("\n"),
        )?;

        Ok(ScriptResult { base_env, output })
    })
}

fn execute_completion(
    dir: &Path,
    shell: Shell,
    command: &str,
    expected: &[&str],
) -> eyre::Result<()> {
    let mut p = spawn_with_options(
        Command::new(shell.to_string()),
        Options {
            timeout_ms: Some(1000),
            strip_ansi_escape_codes: true,
        },
    )?;

    let dir_display = dir.display();
    let bin = escargot::CargoBuild::new()
        .bin("envswitch")
        .current_release()
        .current_target()
        .run()?;
    let bin = bin.path();
    let prefix = shell.script_prefix(bin);

    p.send_line(&prefix)?;
    p.send_line(&format!("cd {dir_display}"))?;
    p.send(command)?;
    // We send two tabs because, on the first one, zsh just completes the
    // current word.
    p.send("\t\t")?;
    p.flush()?;
    p.exp_regex(&expected.join("[ ]+"))?;
    Ok(())
}

pub fn assert_completions(shell: Shell, config: &toml::Table, command: &str, expected: &[&str]) {
    run_test(shell, config, |dir, shell| {
        execute_completion(dir, shell, command, expected)
    })
}
