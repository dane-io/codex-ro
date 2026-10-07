use serde::{Deserialize, Serialize};
use std::{env, fs, io, path::{Path, PathBuf}};


#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub allowed_paths: Vec<PathBuf>,
}


fn config_path() -> io::Result<PathBuf> {
    let home = env::var_os("HOME")
        .ok_or_else(|| io::Error::other("HOME is not set"))?;

    Ok(
        PathBuf::from(home)
            .join(".config")
            .join("codex-ro")
            .join("config.toml"),
    )
}


pub fn load_config() -> io::Result<Config> {
    let path = config_path()?;

    if !path.exists() {
        return Ok(Config::default());
    }

    let contents = fs::read_to_string(&path)?;

    toml::from_str(&contents).map_err(|error| {
        io::Error::other(format!(
            "failed to parse {}: {error}",
            path.display()
        ))
    })
}


pub fn is_project_path_allowed(path: impl AsRef<Path>) -> io::Result<bool> {
    let config = load_config()?;
    let path = fs::canonicalize(path)?;

    Ok(config.allowed_paths.contains(&path))
}


fn save_config(config: &Config) -> io::Result<()> {
    let path = config_path()?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let tmp_path = path.with_extension("tmp");

    let contents = toml::to_string_pretty(config)
        .map_err(|error| io::Error::other(format!("failed to serialize config: {error}")))?;

    fs::write(&tmp_path, contents)?;
    fs::rename(tmp_path, path)?;

    Ok(())
}


pub fn list_allowlist() -> io::Result<()> {
    let config = load_config()?;

    if config.allowed_paths.is_empty() {
        println!("No allowed paths.");
    } else {
        for path in config.allowed_paths {
            println!("{}", path.display());
        }
    }

    Ok(())
}


pub fn add_to_allowlist(path: impl AsRef<Path>) -> io::Result<()> {
    let mut config = load_config()?;
    let path = fs::canonicalize(path)?;

    if !config.allowed_paths.contains(&path) {
        config.allowed_paths.push(path.clone());
        save_config(&config)?;
        println!("Added: {}", path.display());
    }

    Ok(())
}


pub fn remove_from_allowlist(path: impl AsRef<Path>) -> io::Result<()> {
    let mut config = load_config()?;
    let path = fs::canonicalize(path)?;

    if let Some(index) = config.allowed_paths.iter().position(|p| p == &path) {
        config.allowed_paths.remove(index);
        save_config(&config)?;
        println!("Removed: {}", path.display());
    } else {
        println!("Not allowed: {}", path.display());
    }

    Ok(())
}
