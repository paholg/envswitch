use clap::ValueEnum;

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
}

impl Shell {
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
