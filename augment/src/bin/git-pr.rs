use anyhow::{format_err, Result};
use augment::find_git_root;
use git2::Config;
use log::*;
use shared::find_root;
use std::env;

#[derive(structopt::StructOpt)]
enum Args {
  /// Initialize the repo to use git-pr
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
  let mut config = Config::open(&find_git_root()?.join("config"))?;
  for entry in &config.entries(None)? {
    let entry = entry?;
    println!(
      "{} => {}",
      entry
        .name()
        .ok_or_else(|| format_err!("git config entry has no name"))?,
      entry
        .value()
        .ok_or_else(|| format_err!("git config entry has no value"))?
    );
  }
  Ok(())
}
