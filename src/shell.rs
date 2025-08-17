use clap::ValueEnum;

// NOTE: If you add any shells here, make sure to add instructions to the
// readme.
#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum Shell {
    Bash,
    Fish,
    Zsh,
}

impl Shell {
    #[allow(dead_code)]
    fn setup(&self) -> &'static str {
        match self {
            Shell::Bash => "es() { source <(envswitch set -sbash \"$@\"); }",
            Shell::Fish => "function es; envswitch set -sfish $argv | source; end",
            Shell::Zsh => "es() { source <(envswitch set -szsh \"$@\"); }",
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
}
