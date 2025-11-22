use anyhow::Result;
use std::{fs::read_to_string, sync::LazyLock};
use toml::{Table, Value};

pub static ALLOWED_DIRECTORIES: LazyLock<Option<&Vec<String>>> = LazyLock::new(|| {
    PARSED_CONFIG
        .as_ref()
        .map(|config| &config.allowed_directories)
});

pub static ALLOWED_PACKAGES: LazyLock<Option<&Vec<String>>> = LazyLock::new(|| {
    PARSED_CONFIG
        .as_ref()
        .map(|config| &config.allowed_packages)
});

#[derive(Default)]
struct ParsedConfig {
    allowed_directories: Vec<String>,
    allowed_packages: Vec<String>,
}

static PARSED_CONFIG: LazyLock<Option<ParsedConfig>> =
    LazyLock::new(|| allow_or_ignore_table().and_then(|table| parse_table_entries(table)));

/// Determine whether config.toml has an `allow` or `ignore` table.
///
/// A config.toml should have one or the other, but not both.
fn allow_or_ignore_table() -> Option<Table> {
    let config = match read_config() {
        Ok(Some(config)) => config,
        Ok(None) => {
            return None;
        }
        Err(error) => {
            eprintln!("failed to read config.toml: {error}");
            return None;
        }
    };
    let allow_value = config.get("allow");
    let ignore_value = config.get("ignore");
    if allow_value.is_some() && ignore_value.is_some() {
        eprintln!("ignoring config.toml as it contains both `allow` and `ignore` values");
        None
    } else if let Some(allow_value) = allow_value {
        let allow_table = allow_value.as_table();
        if allow_table.is_none() {
            eprintln!("`allow` value is not a table");
        }
        allow_table.cloned()
    } else if let Some(ignore_value) = ignore_value {
        let ignore_table = ignore_value.as_table();
        if ignore_table.is_none() {
            eprintln!("`ignore` value is not a table");
        }
        ignore_table.cloned()
    } else {
        None
    }
}

fn read_config() -> Result<Option<Table>> {
    let base_directories = xdg::BaseDirectories::new();
    let Some(path) = base_directories.find_config_file("build-wrap/config.toml") else {
        return Ok(None);
    };
    let contents = read_to_string(&path)?;
    let table = toml::from_str(&contents)?;
    Ok(Some(table))
}

fn parse_table_entries(table: Table) -> Option<ParsedConfig> {
    let mut parsed_config = ParsedConfig::default();
    for (key, value) in table {
        match key.as_str() {
            "directories" => {
                let array_of_strings = as_array_of_strings("directories", &value)?;
                parsed_config.allowed_directories = array_of_strings;
            }
            "packages" => {
                let array_of_strings = as_array_of_strings("packages", &value)?;
                parsed_config.allowed_packages = array_of_strings;
            }
            _ => {
                eprintln!("Unexpected key: {key}");
                return None;
            }
        }
    }
    Some(parsed_config)
}

fn as_array_of_strings(key: &str, value: &Value) -> Option<Vec<String>> {
    let Some(array) = value.as_array() else {
        eprintln!("`{key}` is not an array");
        return None;
    };
    let mut array_of_strings = Vec::with_capacity(array.len());
    for value in array {
        let Some(s) = value.as_str() else {
            eprintln!("`{key}` value is not a string: {value}");
            return None;
        };
        array_of_strings.push(s.to_owned());
    }
    Some(array_of_strings)
}
