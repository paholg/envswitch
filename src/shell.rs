use std::{env, fmt};

use clap::ValueEnum;

// NOTE: If you add any shells here, make sure to add instructions to the
// readme, and add it to the test cases in this file.
#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum Shell {
    Bash,
    Fish,
    Zsh,
}

impl fmt::Display for Shell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Shell::Bash => "bash",
            Shell::Fish => "fish",
            Shell::Zsh => "zsh",
        };
        name.fmt(f)
    }
}

impl Shell {
    pub fn setup(&self) -> String {
        let bin = env::args().next().unwrap();
        // NOTE: These scripts should use BIN as the binary name for envswitch,
        // which we will sub-in at runtime as a very simple templating
        // mechanism.
        let script = match self {
            Shell::Bash => include_str!("shell/bash_setup.sh"),
            Shell::Fish => include_str!("shell/fish_setup.fish"),
            Shell::Zsh => include_str!("shell/zsh_setup.zsh"),
        };
        script.replace("BIN", &bin)
    }

    pub fn set_var(&self, var: &str, value: &str) -> String {
        match self {
            Shell::Bash | Shell::Zsh => format!("export {var}=\"{value}\""),
            Shell::Fish => format!("set -gx {var} \"{value}\""),
        }
    }

    pub fn clear_var(&self, var: &str) -> String {
        match self {
            Shell::Bash | Shell::Zsh => format!("unset {var}"),
            Shell::Fish => format!("set -e {var}"),
        }
    }
}

#[cfg(test)]
pub mod test {
    use crate::shell::Shell;

    use rstest_reuse::{self, template};

    #[template]
    #[rstest]
    #[case::bash(Shell::Bash)]
    #[case::fish(Shell::Fish)]
    #[case::zsh(Shell::Zsh)]
    pub fn shell_cases(#[case] shell: Shell) {}

    #[template]
    #[rstest]
    #[case::bash(Shell::Bash)]
    #[case::fish(Shell::Fish)]
    // Zsh tests aren't working :(
    // #[case::zsh(Shell::Zsh)]
    pub fn shell_completion_cases(#[case] shell: Shell) {}

    impl Shell {
        pub fn script_prefix(&self, bin: &std::path::Path) -> String {
            let bin = bin.display();
            match self {
                Shell::Bash | Shell::Zsh => {
                    format!("set -euo pipefail; source <({bin} setup {self})")
                }
                Shell::Fish => format!("{bin} setup fish | source"),
            }
        }

        /// Fish doesn't have anything like `set -e`, and we need some way to exit
        /// tests on failure.
        pub fn try_cmd(&self, command: &str) -> String {
            match self {
                // We cover this in the prefix.
                Shell::Bash | Shell::Zsh => command.to_string(),
                Shell::Fish => format!("{command}; or return $status"),
            }
        }

        /// This is an attempt to test completions. It's questionable how well
        /// it works.
        pub fn completion_cmd(&self, args: &[&str]) -> String {
            match self {
                Shell::Bash => {
                    // Simulate bash completion environment and call _es_completion
                    // Use explicit array assignment to avoid parsing issues
                    let setup_cmd = if args.is_empty() {
                        "COMP_WORDS[0]=es COMP_WORDS[1]=\"\" COMP_CWORD=1".to_string()
                    } else {
                        let mut setup = vec!["COMP_WORDS[0]=es".to_string()];
                        for (i, arg) in args.iter().enumerate() {
                            setup.push(format!("COMP_WORDS[{}]=\"{}\"", i + 1, arg));
                        }
                        setup.push(format!("COMP_CWORD={}", args.len()));
                        setup.join(" ")
                    };
                    format!(
                        "{} && _es_completion && printf '%s\\n' \"${{COMPREPLY[@]}}\"",
                        setup_cmd
                    )
                }
                Shell::Zsh => todo!(),
                Shell::Fish => {
                    let mock_commandline = if args.is_empty() {
                        "function commandline; echo es; end".to_string()
                    } else {
                        let mut all_args = vec!["es"];
                        all_args.extend(args.iter().copied());
                        format!(
                            "function commandline; printf '%s\\n' {}; end",
                            all_args
                                .iter()
                                .map(|s| format!("'{}'", s))
                                .collect::<Vec<_>>()
                                .join(" ")
                        )
                    };
                    format!("{} && __es_complete_positional", mock_commandline)
                }
            }
        }
    }
}
