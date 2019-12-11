//! dev-suite cli tool to install and update devsuite and it's tooling
use anyhow::{
  format_err,
  Result,
};
use configamajig::*;
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
  /// Commands for configuration of dev-suite
  Config(Config),
}

#[derive(structopt::StructOpt)]
/// dev-suite config commands
enum Config {
  /// dev-suite config commands for the current user
  User(User),
  /// dev-suite config commands for the current repo
  Repo(Repo),
}

#[derive(structopt::StructOpt)]
enum User {
  /// Initialize the user with a name
  Init { name: String },
  /// Show the current user
  Show,
}

#[derive(structopt::StructOpt)]
enum Repo {
  /// Initialize the repo with a config
  Init,
  /// Show the repo config
  Show,
  /// Add someone as a maintainer
  Add(Add),
}
#[derive(structopt::StructOpt)]
enum Add {
  /// Make self the maintainer
  Me,
}

#[paw::main]
fn main(args: Args) {
  if let Err(e) = match args {
    Args::Init => init(),
    Args::Update => unimplemented!(),
    Args::Install => unimplemented!(),
    Args::Config(conf) => match conf {
      Config::User(user) => match user {
        User::Init { name } => create_user_config(name),
        User::Show => show_user_config(),
      },
      Config::Repo(repo) => match repo {
        Repo::Init => create_repo_config(),
        Repo::Show => show_repo_config(),
        Repo::Add(add) => match add {
          Add::Me => add_self_to_maintainers(),
        },
      },
    },
  } {
    eprintln!("{}", e);
    std::process::exit(1);
  }
}

/// Initialize a git repo with all the tools wanted for it
fn init() -> Result<()> {
  // Make sure we're in a valid git repo
  let _ = find_root()?;
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
    create_repo_config()?;
    add_self_to_maintainers().map_err(|_| {
      format_err!(
      "It looks like this is your first time using dev-suite. Initialize your \
       config with 'ds config user <name>' then rerun 'ds init'."
    )
    })?;
    for selection in selections {
      match selection {
        Tools::Hooked => {
          which("hooked")
            .map(drop)
            .map_err(|_| format_err!(
              "It looks like hooked is not on your $PATH. Did you run 'ds install'?"
            ))?;
          let _ = Command::new("hooked").arg("init").spawn()?.wait()?;
        }
        Tools::Ticket => {
          which("ticket")
            .map(drop)
            .map_err(|_| format_err!(
              "It looks like ticket is not on your $PATH. Did you run 'ds install'?"
            ))?;
          let _ = Command::new("ticket").arg("init").spawn()?.wait()?;
        }
      }
    }
  }
  Ok(())
}

/// Tools available for installation or usage by dev-suite
enum Tools {
  Hooked,
  Ticket,
}
