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

fn lang(lang: &str) -> Result<(), Box<dyn Error>> {
  let dir = tempdir()?;
  let _ = Repository::init(&dir)?;
  let mut cmd = Command::cargo_bin("hooked")?;
  env::set_current_dir(&dir)?;
  let _ = cmd.arg("init").arg(lang).assert().success();
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
    #[cfg(not(windows))]
    assert_eq!(dev_hook.metadata()?.permissions().mode() & 511, 0o755);

    let shebang = fs::read_to_string(&dev_hook)?
      .lines()
      .nth(0)
      .ok_or_else(|| "File is empty and has no shebang line")?
      .to_owned();
    match lang {
      "bash" => assert_eq!(shebang, "#!/usr/bin/env bash"),
      "python" => assert_eq!(shebang, "#!/usr/bin/env python3"),
      "ruby" => assert_eq!(shebang, "#!/usr/bin/env ruby"),
      _ => unreachable!(),
    }
  }
  Ok(())
}

#[test]
fn init_bash() -> Result<(), Box<dyn Error>> {
  lang("bash")
}

#[test]
fn init_python() -> Result<(), Box<dyn Error>> {
  lang("python")
}

#[test]
fn init_ruby() -> Result<(), Box<dyn Error>> {
  lang("ruby")
}
