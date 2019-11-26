use anyhow::{
  format_err,
  Result,
};
use dialoguer::{
  theme::ColorfulTheme,
  Checkboxes,
};
use shared::find_root;
use std::process::Command;
use which::which;

#[derive(structopt::StructOpt)]
enum Args {
  /// Download and install all of dev-suite
  Install,
  /// Update all of dev-suite
  Update,
  /// Initialize the repo to use dev-suite and it's tools
  Init,
}

#[paw::main]
fn main(args: Args) {
  if let Err(e) = match args {
    Args::Init => init(),
    Args::Update => unimplemented!(),
    Args::Install => unimplemented!(),
  } {
    eprintln!("{}", e);
    std::process::exit(1);
  }
}

fn init() -> Result<()> {
  // Make sure we're in a valid git repo
  find_root()?;
  let checkboxes = &["hooked - Managed git hooks", "ticket - In repo tickets"];
  let defaults = &[true, true];
  let selections = Checkboxes::with_theme(&ColorfulTheme::default())
    .with_prompt("Which tools do you want to enable? (defaults to all)")
    .items(&checkboxes[..])
    .defaults(&defaults[..])
    .interact()?
    .into_iter()
    .map(|s| match s {
      0 => Tools::Hooked,
      1 => Tools::Ticket,
      _ => unreachable!(),
    })
    .collect::<Vec<Tools>>();

  if selections.is_empty() {
    println!("Nothing selected. dev-suite not enabled in this repository.");
  } else {
    for selection in selections {
      match selection {
        Tools::Hooked => {
          which("hooked")
            .map(drop)
            .map_err(|_| format_err!(
              "It looks like hooked is not on your $PATH. Did you run 'ds install'?"
            ))?;
          Command::new("hooked").arg("init").spawn()?.wait()?;
        }
        Tools::Ticket => {
          which("ticket")
            .map(drop)
            .map_err(|_| format_err!(
              "It looks like ticket is not on your $PATH. Did you run 'ds install'?"
            ))?;
          Command::new("ticket").arg("init").spawn()?.wait()?;
        }
      }
    }
  }
  Ok(())
}

enum Tools {
  Hooked,
  Ticket,
}
