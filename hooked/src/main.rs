#[cfg(windows)]
use anyhow::bail;
use anyhow::Result;
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
  /// Initialize the repo to use ticket
  Init,
}

#[paw::main]
fn main(args: Args) {
  env::var("RUST_LOG").map(drop).unwrap_or_else(|_| {
    env::set_var("RUST_LOG", "info");
  });
  pretty_env_logger::init();
  if let Err(e) = match args {
    Args::Init => init(),
  } {
    error!("{}", e);
    std::process::exit(1);
  }
}

fn init() -> Result<()> {
  #[cfg(windows)]
  bail!("Windows is currently unsupported!");

  let root = find_root()?;
  let git_hooks = &root.join(".git").join("hooks");
  debug!("git_hooks base path: {}", git_hooks.display());
  let root = root.join(".dev-suite").join("hooked");
  debug!("root base path: {}", root.display());
  fs::create_dir_all(&root)?;

  for hook in &HOOKS {
    let path = &root.join(hook);
    debug!("dev-suite hook path: {}", path.display());
    let git_hook = &git_hooks.join(hook);
    debug!("git_hook path: {}", git_hook.display());

    if !path.exists() {
      debug!("Creating dev-suite hook.");
      let mut file = fs::File::create(&path)?;
      trace!("File created.");
      let mut perms = file.metadata()?.permissions();
      debug!("Setting dev-suite hook to be executable.");
      perms.set_mode(0o755);
      file.set_permissions(perms)?;
      trace!("Permissions were set.");
      file.write_all(b"#! /bin/bash")?;
      debug!("Writing data to file.");
      debug!("Created git hook {}.", hook);
    } else {
      debug!("git hook {} already exists. Skipping creation.", hook);
    }
    let path = path.canonicalize()?;

    if !git_hook.exists() {
      debug!("Symlinking git hook {}.", hook);
      #[cfg(not(windows))]
      symlink(&path, &git_hook)?;
      #[cfg(windows)]
      symlink_file(&path, &git_hook)?;
      trace!("Symlinked git hook {} to .dev-suite/hooked/{}.", hook, hook);
    }
  }
  info!(
    "Created and symlinked tickets to .git/hooks from {}.",
    root.display()
  );

  Ok(())
}
