//! git hook manager tool

use anyhow::{
  bail,
  Result,
};
use log::*;
use shared::find_root;
#[cfg(not(windows))]
use std::os::unix::fs::{
  symlink,
  PermissionsExt,
};
#[cfg(windows)]
use std::os::windows::fs::symlink_file;
use std::{
  env,
  fs,
  io::Write,
  path::Path,
};

const HOOKS: [&str; 18] = [
  "applypatch-msg",
  "post-applypatch",
  "pre-commit",
  "prepare-commit-msg",
  "commit-msg",
  "post-commit",
  "pre-rebase",
  "post-checkout",
  "post-merge",
  "pre-push",
  "pre-receive",
  "update",
  "post-receive",
  "post-update",
  "push-to-checkout",
  "pre-auto-gc",
  "post-rewrite",
  "sendemail-validate",
];

#[derive(structopt::StructOpt)]
enum Args {
  /// Initialize the repo to use hooked
  Init(Language),
  /// Link pre existing hooks to your .git folder
  Link,
}

/// Which language the repo should be initialized with for hooks
#[derive(Clone, Copy, structopt::StructOpt)]
enum Language {
  /// Use Bash for your git hooks
  Bash,
  /// Use Python 3 for your git hooks
  Python,
  /// Use Ruby for your git hooks
  Ruby,
}

#[paw::main]
fn main(args: Args) {
  env::var("RUST_LOG")
    .ok()
    .map_or_else(|| env::set_var("RUST_LOG", "info"), drop);
  pretty_env_logger::init();
  if let Err(e) = match args {
    Args::Init(lang) => init(lang),
    Args::Link => link(),
  } {
    error!("{}", e);
    std::process::exit(1);
  }
}

fn init(lang: Language) -> Result<()> {
  let root = find_root()?;
  let git_hooks = &root.join(".git").join("hooks");
  debug!("git_hooks base path: {}", git_hooks.display());
  let root = root.join(".dev-suite").join("hooked");
  debug!("root base path: {}", root.display());
  let wrapper_dir = &root.join("wrapper");
  fs::create_dir_all(&wrapper_dir)?;

  for hook in &HOOKS {
    let mut path = (&root).join(hook);
    debug!("dev-suite hook path: {}", path.display());
    let git_hook = &git_hooks.join(hook);
    debug!("git_hook path: {}", git_hook.display());
    let mut wrapper_hook = (&wrapper_dir).join(hook);
    let _ = wrapper_hook.set_extension("sh");
    let _ = match lang {
      Language::Bash => path.set_extension("sh"),
      Language::Python => path.set_extension("py"),
      Language::Ruby => path.set_extension("rb"),
    };

    if path.exists() {
      debug!("git hook {} already exists. Skipping creation.", hook);
    } else {
      debug!("Creating dev-suite hook.");
      let mut file = fs::File::create(&path)?;
      let mut wrapper = fs::File::create(&wrapper_hook)?;
      trace!("File created.");
      #[cfg(not(windows))]
      {
        let mut perms = file.metadata()?.permissions();
        let mut wperms = wrapper.metadata()?.permissions();
        debug!("Setting dev-suite hook to be executable.");
        perms.set_mode(0o755);
        wperms.set_mode(0o755);
        file.set_permissions(perms)?;
        wrapper.set_permissions(wperms)?;
        trace!("Permissions were set.");
      }
      match lang {
        Language::Bash => {
          file.write_all(b"#!/usr/bin/env bash")?;
          wrapper.write_all(
            format!(
              "#!C:\\Program Files\\Git\\bin\\sh.exe\n\
              bash.exe .dev-suite/hooked/{}.sh\n",
              hook
            )
            .as_bytes(),
          )?;
        }
        Language::Python => {
          file.write_all(b"#!/usr/bin/env python3")?;
          wrapper.write_all(
            format!(
              "#!C:\\Program Files\\Git\\bin\\sh.exe\n\
              py.exe .dev-suite/hooked/{}.py\n",
              hook
            )
            .as_bytes(),
          )?;
        }
        Language::Ruby => {
          file.write_all(b"#!/usr/bin/env ruby")?;
          wrapper.write_all(
            format!(
              "#!C:\\Program Files\\Git\\bin\\sh.exe\n\
              ruby.exe .dev-suite/hooked/{}.rb\n",
              hook
            )
            .as_bytes(),
          )?;
        }
      }
      debug!("Writing data to file.");
      debug!("Created git hook {}.", hook);
    }
    let path = path.canonicalize()?;
    inner_link(&path, &git_hook, hook)?;
  }
  info!(
    "Created and symlinked tickets to .git/hooks from {}.",
    root.display()
  );

  Ok(())
}

fn link() -> Result<()> {
  let root = find_root()?;
  let git_hooks = &root.join(".git").join("hooks");
  debug!("git_hooks base path: {}", git_hooks.display());
  let root = root.join(".dev-suite").join("hooked");
  debug!("root base path: {}", root.display());

  for hook in &HOOKS {
    let path = {
      #[cfg(windows)]
      let mut path = root.join("wrapper").join(hook);
      #[cfg(not(windows))]
      let mut path = root.join(hook);

      let _ = path.set_extension("py");
      if path.exists() {
        path
      } else if {
        let _ = path.set_extension("rb");
        path.exists()
      } {
        path
      } else if {
        let _ = path.set_extension("sh");
        path.exists()
      } {
        path
      } else {
        let _ = path.set_extension("");
        bail!(
        "The path {} does not exist. Have you initialized the repo to use hooked?",
        path.display()
      );
      }
    };
    if path.exists() {
      let path = path.canonicalize()?;
      debug!("dev-suite hook path: {}", path.display());
      let git_hook = &git_hooks.join(hook);
      debug!("git_hook path: {}", git_hook.display());
      inner_link(&path, &git_hook, hook)?;
    }
  }

  info!("Successfully symlinked all githooks to .git/hooks");
  Ok(())
}

fn inner_link(path: &Path, git_hook: &Path, hook: &str) -> Result<()> {
  if !git_hook.exists() {
    debug!("Symlinking git hook {}.", hook);
    #[cfg(not(windows))]
    symlink(&path, &git_hook)?;
    #[cfg(windows)]
    symlink_file(&path, &git_hook)?;
    trace!("Symlinked git hook {} to .dev-suite/hooked/{}.", hook, hook);
  }
  Ok(())
}
