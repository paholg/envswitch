use std::ops::Deref;

use eyre::eyre;
use indexmap::IndexMap;
use itertools::Itertools;

use crate::{
    config::{Key, Table},
    shell::Shell,
};

#[derive(Debug, Default)]
pub struct ConfigWalker<'a> {
    pub vals: IndexMap<&'a str, &'a str>,
}

impl<'a> ConfigWalker<'a> {
    pub fn new(config: &'a Table, keys: impl Iterator<Item = &'a Key>) -> eyre::Result<Self> {
        let mut this = Self::default();
        this.walk(config, keys)?;
        Ok(this)
    }

    pub fn set_commands(&self, shell: &Shell) -> impl Iterator<Item = String> {
        self.vals
            .iter()
            .map(|(var, value)| shell.set_var(var, value))
    }

    pub fn variables(&self) -> String {
        Itertools::intersperse(self.vals.keys().map(Deref::deref), " ").collect()
    }

    fn walk(
        &mut self,
        config: &'a Table,
        mut keys: impl Iterator<Item = &'a Key>,
    ) -> eyre::Result<()> {
        // First we track any variables that are set at this level:
        let variables = config
            .iter()
            .flat_map(|(k, v)| v.as_string().map(|v| (k, v)));
        for (var, value) in variables {
            self.vals.insert(&var, value);
        }

        // Now we go to the next level:
        let Some(head) = keys.next() else {
            return Ok(());
        };
        if head.is_empty() {
            return Ok(());
        }
        let missing_key = || {
            let options = config
                .iter()
                .filter_map(|(k, v)| v.as_table().map(|_| k))
                .join(", ");
            eyre!("missing key '{head}'. Available options: {options}")
        };
        let inner = config
            .get(head)
            .ok_or_else(missing_key)?
            .as_table()
            .ok_or_else(|| eyre!("key '{head}' does not correspond to a table"))?;

        self.walk(inner, keys)?;

        Ok(())
    }
}
