use assert_cmd::prelude::*;
use git2::Repository;
use std::{
  env,
  error::Error,
  fs,
  os::unix::fs::PermissionsExt,
  process::Command,
};
use tempfile::tempdir;

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

#[test]
fn init() -> Result<(), Box<dyn Error>> {
  let dir = tempdir()?;
  let _ = Repository::init(&dir)?;
  let mut cmd = Command::cargo_bin("hooked")?;
  env::set_current_dir(&dir)?;
  let _ = cmd.arg("init").assert().success();
  let git = &dir.path().join(".git").join("hooks");
  let dev = &dir.path().join(".dev-suite").join("hooked");

  for hook in &HOOKS {
    let git_hook = git.join(hook);
    let dev_hook = dev.join(hook);
    assert!(&git_hook.exists());
    assert!(&dev_hook.exists());
    assert!(fs::symlink_metadata(&git_hook)?.file_type().is_symlink());
    assert!(dev_hook.is_file());
    // why are we doing a bitwise and?
    // git does a thing where it'll tack on extra bits to make the mode bit
    // 0o100755 which is for uncommitted files or something. Not great. 511 is
    // just 9 1s in binary with the rest being zero. This lets us get 3 octal
    // numbers essentially, allowing us to test the actual value we wanted to
    // test, and making this test work without special casing it if git ever
    // changes.
    assert_eq!(dev_hook.metadata()?.permissions().mode() & 511, 0o755);
  }

  Ok(())
}
