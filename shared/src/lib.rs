//! Common functionality needed in many of the tools

use anyhow::{
  bail,
  Result,
};
use std::{
  env,
  path::PathBuf,
};

/// Finds the top level folder of the repo and returns it's canonicalized path
pub fn find_root() -> Result<PathBuf> {
  let mut location = env::current_dir()?;
  let mut found_root = false;

  for loc in location.ancestors() {
    let mut loc = loc.join(".git");
    if loc.exists() {
      let _ = loc.pop();
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
