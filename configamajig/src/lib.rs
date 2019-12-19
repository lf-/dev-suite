//! config management lib for dev-suite
use anyhow::{
  format_err,
  Result,
};
use dirs::config_dir;
use serde::{
  Deserialize,
  Serialize,
};
use shared::find_root;
use std::{
  fs,
  path::PathBuf,
};
use uuid::Uuid;

/// Creates a new user config if it does not exist
pub fn create_user_config(name: impl Into<String>) -> Result<()> {
  let conf_dir = config_dir()
    .ok_or_else(|| format_err!("Unable to get the config dir for the OS"))?
    .join("dev-suite");
  if !conf_dir.exists() {
    fs::create_dir_all(&conf_dir)?;
  }
  let conf_path = conf_dir.join("user-config.toml");
  if !conf_path.exists() {
    let user_config = UserConfig::new(name);
    fs::write(&conf_path, toml::to_string_pretty(&user_config)?)?;
  }
  Ok(())
}

/// Creates a new repo config if it does not exist
pub fn create_repo_config() -> Result<()> {
  let conf_dir = find_root()?.join(".dev-suite");
  if !conf_dir.exists() {
    fs::create_dir_all(&conf_dir)?;
  }
  let conf_path = conf_dir.join("repo-config.toml");
  if !conf_path.exists() {
    let repo_config = RepoConfig::new();
    fs::write(&conf_path, toml::to_string_pretty(&repo_config)?)?;
  }
  Ok(())
}

/// Get the path for the user config
fn user_config_path() -> Result<PathBuf> {
  Ok(
    config_dir()
      .ok_or_else(|| format_err!("Unable to get the config dir for the OS"))?
      .join("dev-suite")
      .join("user-config.toml"),
  )
}

/// Get the path for the repo config
fn repo_config_path() -> Result<PathBuf> {
  Ok(find_root()?.join(".dev-suite").join("repo-config.toml"))
}

/// Reads in the user config
pub fn get_user_config() -> Result<UserConfig> {
  Ok(toml::from_slice(&fs::read(&user_config_path()?)?)?)
}
/// Reads in the repo config
pub fn get_repo_config() -> Result<RepoConfig> {
  Ok(toml::from_slice(&fs::read(&repo_config_path()?)?)?)
}

/// Writes the user config to disk
// We don't want the user to use the old value again
#[allow(clippy::needless_pass_by_value)]
pub fn set_user_config(user_config: UserConfig) -> Result<()> {
  fs::write(&user_config_path()?, toml::to_string_pretty(&user_config)?)?;
  Ok(())
}

/// Writes the user config to disk
// We don't want the user to use the old value again
#[allow(clippy::needless_pass_by_value)]
pub fn set_repo_config(repo_config: RepoConfig) -> Result<()> {
  fs::write(&repo_config_path()?, toml::to_string_pretty(&repo_config)?)?;
  Ok(())
}

/// User Config struct
#[derive(Serialize, Deserialize, Debug)]
pub struct UserConfig {
  /// The name of the user using dev-suite
  pub name: String,
  /// The uuid of the user using dev-suite
  pub uuid: Uuid,
}

impl UserConfig {
  /// Create a new `UserConfig` from a given name and assign UUID to them
  pub fn new(name: impl Into<String>) -> Self {
    Self {
      name: name.into(),
      uuid: Uuid::new_v4(),
    }
  }
}

/// Repo Config struct
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct RepoConfig {
  maintainers: Vec<(String, Uuid)>,
}

impl RepoConfig {
  /// Create a new `RepoConfig`
  #[must_use]
  pub fn new() -> Self {
    Self {
      maintainers: Vec::new(),
    }
  }
}

/// Show repo config
pub fn show_repo_config() -> Result<()> {
  let conf = get_repo_config()?;
  for m in conf.maintainers {
    println!("{} - {}", m.0, m.1);
  }
  Ok(())
}

/// Show repo config
pub fn show_user_config() -> Result<()> {
  let conf = get_user_config()?;
  println!("{} - {}", conf.name, conf.uuid);
  Ok(())
}

/// Add current user to this repo's list of maintainers
pub fn add_self_to_maintainers() -> Result<()> {
  let mut repo_conf = get_repo_config()?;
  let user_conf = get_user_config()?;
  if repo_conf
    .maintainers
    .iter()
    .any(|x| x.0 == user_conf.name && x.1 == user_conf.uuid)
  {
    Ok(())
  } else {
    repo_conf.maintainers.push((user_conf.name, user_conf.uuid));
    set_repo_config(repo_conf)
  }
}
