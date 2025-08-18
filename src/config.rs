use std::{fmt, ops::Deref};

use indexmap::IndexMap;
use serde::{
    Deserialize,
    de::{self, Visitor},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Key(String);

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Deref for Key {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for Key {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

const FORBIDDEN_CHARS: &[char] = &['.', ',', ':', ' '];

impl TryFrom<String> for Key {
    type Error = eyre::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match FORBIDDEN_CHARS
            .iter()
            .filter_map(|&ch| value.contains(ch).then_some(ch))
            .next()
        {
            Some(invalid_char) => Err(eyre::eyre!("Config keys cannot contain '{invalid_char}'")),
            None => Ok(Key(value)),
        }
    }
}

impl<'de> Deserialize<'de> for Key {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Key::try_from(s).map_err(serde::de::Error::custom)
    }
}

pub type Table = IndexMap<Key, Value>;

#[derive(Debug)]
pub enum Value {
    String(String),
    Table(Table),
}

pub fn deep_keys(table: &Table) -> impl Iterator<Item = String> {
    fn collect_keys(table: &Table, prefix: &str, keys: &mut Vec<String>) {
        for (key, value) in table {
            let Some(table) = value.as_table() else {
                continue;
            };
            let full_key = if prefix.is_empty() {
                key.0.clone()
            } else {
                format!("{}.{}", prefix, key)
            };

            keys.push(full_key.clone());

            collect_keys(table, &full_key, keys);
        }
    }

    let mut keys = Vec::new();
    collect_keys(table, "", &mut keys);
    keys.into_iter()
}

impl<'de> Deserialize<'de> for Value {
    // We implement this ourselves instead of using `#[serde(untagged)]` to get
    // better error messages.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ValueVisitor;

        impl<'de> Visitor<'de> for ValueVisitor {
            type Value = Value;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string or table")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::String(v.to_owned()))
            }

            fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                Table::deserialize(de::value::MapAccessDeserializer::new(map)).map(Value::Table)
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}

impl Value {
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Value::String(val) => Some(val),
            _ => None,
        }
    }

    pub fn as_table(&self) -> Option<&Table> {
        match self {
            Value::Table(val) => Some(val),
            _ => None,
        }
    }
}
