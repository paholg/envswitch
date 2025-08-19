use std::sync::LazyLock;

use indexmap::indexmap;
use rstest::rstest;
use rstest_reuse::apply;

use crate::shell::Shell;
use crate::shell::test::{shell_cases, shell_completion_cases};
use crate::test::helpers::{get_completions, run_command};

mod helpers;

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

#[apply(shell_cases)]
fn staging(#[case] shell: Shell) {
    let r = run_command(shell, &CONFIG, "es staging");
    assert_eq!(r.status(), 0);

    assert_eq!(
        r.env_diff(),
        indexmap! {
            "ENVSWITCH_ENV" => "staging:GLOBAL,URL",
            "GLOBAL" => "some global variable",
            "URL" => "staging.com",
        }
    );
}

#[apply(shell_cases)]
fn staging_abc(#[case] shell: Shell) {
    let r = run_command(shell, &CONFIG, "es staging.abc");
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
}

#[apply(shell_cases)]
fn staging_def(#[case] shell: Shell) {
    let r = run_command(shell, &CONFIG, "es staging.def");
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
}

#[apply(shell_cases)]
fn prod_abc(#[case] shell: Shell) {
    let r = run_command(shell, &CONFIG, "es prod.abc");
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
}

#[apply(shell_cases)]
fn prod(#[case] shell: Shell) {
    let r = run_command(shell, &CONFIG, &["es prod.abc", "es prod"].join("\n"));
    assert_eq!(r.status(), 0);

    r.assert_stderr_includes("Environment set: prod.abc ");
    r.assert_stderr_includes("Environment set: prod ");

    assert_eq!(
        r.env_diff(),
        indexmap! {
            "ENVSWITCH_ENV" => "prod:GLOBAL,URL",
            "GLOBAL" => "override for production",
            "URL" => "prod.com",
        }
    );
}

#[apply(shell_cases)]
fn base(#[case] shell: Shell) {
    let r = run_command(shell, &CONFIG, &["es prod", "es"].join("\n"));
    assert_eq!(r.status(), 0);

    r.assert_stderr_includes("Environment set: prod ");

    assert_eq!(
        r.env_diff(),
        indexmap! {
            "ENVSWITCH_ENV" => ":GLOBAL",
            "GLOBAL" => "some global variable",
        }
    );
}

#[apply(shell_cases)]
fn missing_file(#[case] shell: Shell) {
    let r = run_command(shell, &CONFIG, "es -f foo.toml");
    assert_ne!(r.status(), 0);

    assert!(r.env_diff().is_empty());
}

#[apply(shell_cases)]
fn list(#[case] shell: Shell) {
    let r = run_command(shell, &CONFIG, "es -l");
    assert_eq!(r.status(), 0);

    assert!(r.env_diff().is_empty());

    r.assert_stderr_includes("Available environments:");
    r.assert_stderr_includes("staging\n");
    r.assert_stderr_includes("staging.abc\n");
    r.assert_stderr_includes("staging.def\n");
    r.assert_stderr_includes("prod\n");
    r.assert_stderr_includes("prod.abc\n");
}

#[apply(shell_cases)]
fn bad_command(#[case] shell: Shell) {
    let r = run_command(shell, &CONFIG, "es -g");
    assert_ne!(r.status(), 0);

    assert!(r.env_diff().is_empty());
    r.assert_stderr_includes("error: unexpected argument '-g' found");
}

#[apply(shell_cases)]
fn help(#[case] shell: Shell) {
    let r = run_command(shell, &CONFIG, "es -h");
    assert_ne!(r.status(), 0);

    assert!(r.env_diff().is_empty());

    r.assert_stderr_includes("error: unexpected argument '-h' found");
}

#[apply(shell_completion_cases)]
fn completion_empty(#[case] shell: Shell) {
    let r = get_completions(shell, &CONFIG, &[]);
    assert_eq!(r.status(), 0);

    let completions = r.completions();
    assert_eq!(
        completions,
        ["prod", "prod.abc", "staging", "staging.abc", "staging.def"]
    );
}

#[apply(shell_completion_cases)]
fn completion_partial(#[case] shell: Shell) {
    let r = get_completions(shell, &CONFIG, &["stag"]);
    assert_eq!(r.status(), 0);

    let completions = r.completions();
    assert_eq!(completions, ["staging", "staging.abc", "staging.def"]);
}

#[apply(shell_completion_cases)]
fn completion_with_file(#[case] shell: Shell) {
    let r = get_completions(shell, &CONFIG, &["-f", "envswitch.toml", "prod"]);
    assert_eq!(r.status(), 0);

    let completions = r.completions();
    assert_eq!(completions, ["prod", "prod.abc"]);
}

#[apply(shell_completion_cases)]
fn completion_with_file_and_list(#[case] shell: Shell) {
    let r = get_completions(shell, &CONFIG, &["-f", "envswitch.toml", "-l", "prod"]);
    assert_eq!(r.status(), 0);

    let completions = r.completions();
    assert_eq!(completions, ["prod", "prod.abc"]);
}

#[apply(shell_completion_cases)]
fn completion_full(#[case] shell: Shell) {
    let r = get_completions(shell, &CONFIG, &["staging.abc", ""]);
    assert_eq!(r.status(), 0);

    let completions = r.completions();
    let rhs: &[&str] = &[];
    assert_eq!(completions, rhs);
}

// NOTE: These tests fail for fish as our test setup only tests the positional
// complete function.
// #[apply(shell_completion_cases)]
// fn completion_flag(#[case] shell: Shell) {
//     let r = get_completions(shell, &CONFIG, &["-"]);
//     assert_eq!(r.status(), 0);

//     let completions = r.completions();
//     assert_eq!(completions, ["-f", "--file", "-l", "--list"]);
// }

// #[apply(shell_completion_cases)]
// fn completion_file(#[case] shell: Shell) {
//     let r = get_completions(shell, &CONFIG, &["-f", ""]);
//     assert_eq!(r.status(), 0);

//     let completions = r.completions();
//     assert_eq!(completions, ["completion_script", "envswitch.toml"]);
// }
