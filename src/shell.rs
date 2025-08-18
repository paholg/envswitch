use std::{env, fmt};

use clap::ValueEnum;
use indoc::formatdoc;

// NOTE: If you add any shells here, make sure to add instructions to the
// readme.
#[cfg_attr(test, derive(strum::EnumIter))]
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
        match self {
            Shell::Bash | Shell::Zsh => formatdoc! {"
                es() {{
                    local env
                    env=$({bin} set -s{self} \"$@\") || return $?
                    source <(echo \"$env\")
                }}
            "},
            Shell::Fish => formatdoc! {"
                function es
                    {bin} set -s{self} $argv | source
                    return $pipestatus[1]
                end
            "},
        }
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

    pub fn as_clap_complete(&self) -> clap_complete::Shell {
        match self {
            Shell::Bash => clap_complete::Shell::Bash,
            Shell::Fish => clap_complete::Shell::Fish,
            Shell::Zsh => clap_complete::Shell::Zsh,
        }
    }

    #[cfg(test)]
    pub fn script_prefix(&self, bin: &std::path::Path) -> String {
        use indoc::formatdoc;

        let bin = bin.display();
        match self {
            Shell::Bash | Shell::Zsh => formatdoc! {"
                #!/usr/bin/env {self}
                set -euo pipefail

                source <({bin} setup bash)
                "},
            Shell::Fish => formatdoc! {"
                #!/usr/bin/env fish

                {bin} setup fish | source
                "},
        }
    }

    /// Fish doesn't have anything like `set -e`, and we need some way to exit
    /// tests on failure.
    #[cfg(test)]
    pub fn try_cmd(&self, command: &str) -> String {
        match self {
            // We cover this in the prefix.
            Shell::Bash | Shell::Zsh => command.to_string(),
            Shell::Fish => format!("{command}; or return $status"),
        }
    }
}
