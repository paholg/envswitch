use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

use eyre::{Context, eyre};
use indexmap::IndexMap;
use indoc::formatdoc;
use strum::IntoEnumIterator;

use crate::shell::Shell;

struct ScriptResult {
    base_env: String,
    output: Output,
}

fn parse_env(env: &str) -> IndexMap<&str, &str> {
    env.lines()
        .filter_map(|line| line.split_once('='))
        .collect()
}

impl ScriptResult {
    fn status(&self) -> i32 {
        self.output.status.code().unwrap()
    }

    fn env_diff(&self) -> IndexMap<&str, &str> {
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

    fn assert_stderr_includes(&self, s: &str) {
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
    let script_body = formatdoc! {"
        {prefix}

        cd {dir_display}

        {command}

        sh -c env
    "};
    println!("SCRIPT BODY:\n{script_body}");

    let script_path = dir.join(name);
    fs::write(&script_path, script_body.as_bytes())?;
    let output = Command::new(shell.to_string()).arg(&script_path).output()?;
    Ok(output)
}

fn run_inner(
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

fn run<F>(config: &toml::Table, command: &str, assert: F)
where
    F: Fn(ScriptResult),
{
    let _ = color_eyre::install();

    for shell in Shell::iter() {
        println!("Running for {shell}");
        let dir = tempfile::tempdir().unwrap().keep();
        let result = run_inner(config, shell, &dir, command);
        // We opted out of automatic cleanup, so we need to be sure to clean up after ourselves.
        std::fs::remove_dir_all(dir).unwrap();

        assert(result.unwrap());
    }
}
mod readme {
    use indexmap::indexmap;

    use std::sync::LazyLock;

    use crate::test::run;

    static CONFIG: LazyLock<toml::Table> = LazyLock::new(|| {
        toml::toml! {
            GLOBAL = "some global variable"

            [staging]
            URL = "staging.com"

            [staging.abc]
            KEY = "secret_ABC"

            [staging.def]
            KEY = "secret_DEF"
            URL = "def.staging.com"

            [prod]
            GLOBAL = "override for production"
            URL    = "prod.com"

            [prod.abc]
            KEY = "prod_secret_ABC"
        }
    });

    #[test]
    fn staging() {
        run(&CONFIG, "es staging", |r| {
            assert_eq!(r.status(), 0);
            assert_eq!(
                r.env_diff(),
                indexmap! {
                    "ENVSWITCH_ENV" => "staging:GLOBAL,URL",
                    "GLOBAL" => "some global variable",
                    "URL" => "staging.com",
                }
            );
        })
    }

    #[test]
    fn staging_abc() {
        run(&CONFIG, "es staging.abc", |r| {
            assert_eq!(r.status(), 0);
            assert_eq!(
                r.env_diff(),
                indexmap! {
                    "ENVSWITCH_ENV" => "staging.abc:GLOBAL,URL,KEY",
                    "GLOBAL" => "some global variable",
                    "URL" => "staging.com",
                    "KEY" => "secret_ABC",
                }
            );
        })
    }

    #[test]
    fn staging_def() {
        run(&CONFIG, "es staging.def", |r| {
            assert_eq!(r.status(), 0);
            assert_eq!(
                r.env_diff(),
                indexmap! {
                    "ENVSWITCH_ENV" => "staging.def:GLOBAL,URL,KEY",
                    "GLOBAL" => "some global variable",
                    "URL" => "def.staging.com",
                    "KEY" => "secret_DEF",
                }
            );
        })
    }

    #[test]
    fn prod_abc() {
        run(&CONFIG, "es prod.abc", |r| {
            assert_eq!(r.status(), 0);
            assert_eq!(
                r.env_diff(),
                indexmap! {
                    "ENVSWITCH_ENV" => "prod.abc:GLOBAL,URL,KEY",
                    "GLOBAL" => "override for production",
                    "URL" => "prod.com",
                    "KEY" => "prod_secret_ABC",
                }
            );
        })
    }

    #[test]
    fn prod() {
        run(&CONFIG, "es prod.abc\nes prod", |r| {
            r.assert_stderr_includes("Environment set: prod.abc ");
            r.assert_stderr_includes("Environment set: prod ");
            assert_eq!(r.status(), 0);
            assert_eq!(
                r.env_diff(),
                indexmap! {
                    "ENVSWITCH_ENV" => "prod:GLOBAL,URL",
                    "GLOBAL" => "override for production",
                    "URL" => "prod.com",
                }
            );
        })
    }

    #[test]
    fn base() {
        run(&CONFIG, "es prod\nes", |r| {
            assert_eq!(r.status(), 0);
            r.assert_stderr_includes("Environment set: prod ");
            assert_eq!(
                r.env_diff(),
                indexmap! {
                    "ENVSWITCH_ENV" => ":GLOBAL",
                    "GLOBAL" => "some global variable",
                }
            );
        })
    }

    #[test]
    fn missing_file() {
        run(&CONFIG, "es -f foo.toml", |r| {
            assert_ne!(r.status(), 0);
            assert!(r.env_diff().is_empty());
        })
    }
}
