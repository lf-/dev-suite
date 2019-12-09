use anyhow::{bail, Result};
use std::{env, path::PathBuf};

pub fn find_git_root() -> Result<PathBuf> {
  let mut location = env::current_dir()?;
  let mut found_root = false;

  for loc in location.ancestors() {
    let mut loc = loc.join(".git");
    if loc.exists() {
      found_root = true;
      location = loc.canonicalize()?;
      break;
    }
  }

  if found_root {
    Ok(location)
  } else {
    bail!("Unable to find a valid git repo");
  }
}
