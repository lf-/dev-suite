#[cfg(windows)]
use anyhow::bail;
use anyhow::Result;
use shared::find_root;
#[cfg(not(windows))]
use std::os::unix::fs::{
  symlink,
  PermissionsExt,
};
#[cfg(windows)]
use std::os::windows::fs::symlink_file;
use std::{
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
  if let Err(e) = match args {
    Args::Init => init(),
  } {
    eprintln!("{}", e);
    std::process::exit(1);
  }
}

fn init() -> Result<()> {
  #[cfg(windows)]
  bail!("Windows is currently unsupported!");

  let root = find_root()?;
  let git_hooks = &root.join(".git").join("hooks");
  let root = root.join(".dev-suite").join("hooked");
  fs::create_dir_all(&root)?;

  for hook in &HOOKS {
    let path = &root.join(hook);
    let git_hook = &git_hooks.join(hook);

    if !path.exists() {
      let mut file = fs::File::create(&path)?;
      let mut perms = file.metadata()?.permissions();
      perms.set_mode(0o755);
      file.set_permissions(perms)?;
      file.write_all(b"#! /bin/bash")?;
    }
    let path = path.canonicalize()?;

    if !git_hook.exists() {
      #[cfg(not(windows))]
      symlink(&path, &git_hook)?;
      #[cfg(windows)]
      symlink_file(&path, &git_hook)?;
    }
  }

  Ok(())
}
