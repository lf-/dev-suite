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
#[cfg(target_family = "unix")]
use std::os::unix::fs::OpenOptionsExt;
#[cfg(target_os = "macos")]
use std::path::PathBuf;
use std::{
  fs::{
    create_dir_all,
    OpenOptions,
  },
  process::Command,
};
use which::which;

#[derive(structopt::StructOpt)]
enum Args {
  /// Download and install all of dev-suite
  Install,
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
    Args::Install => install(),
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

/// Install all of dev-suite
fn install() -> Result<()> {
  static BASE_URL: &str =
    "https://dev-suite-spaces.nyc3.digitaloceanspaces.com/";

  #[cfg(target_os = "macos")]
  static TOOLS: [&str; 2] = ["hooked_osx", "ticket_osx"];
  #[cfg(target_os = "linux")]
  static TOOLS: [&str; 2] = ["hooked_linux", "ticket_linux"];
  #[cfg(target_os = "windows")]
  static TOOLS: [&str; 2] = ["hooked_windows", "ticket_windows"];

  #[cfg(target_os = "macos")]
  let mut location = PathBuf::from("/usr/local/bin");
  #[cfg(target_os = "linux")]
  let mut location = dirs::executable_dir().unwrap();
  #[cfg(target_os = "windows")]
  let mut location = dirs::data_local_dir().unwrap().join("dev-suite");

  let client = reqwest::blocking::Client::new();
  create_dir_all(&location)?;

  for tool in &TOOLS {
    let tool_name = tool.split('_').nth(0).unwrap();
    location.push(tool_name);
    if location.exists() {
      println!("{} already exists, skipping", tool_name);
      let _ = location.pop();
    } else {
      println!("Installing {}", tool_name);
      let url = BASE_URL.to_owned() + tool;
      let mut program = client.get(&url).send()?;

      #[cfg(target_family = "unix")]
      let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .mode(0o755)
        .open(&location)?;
      #[cfg(target_family = "windows")]
      let mut file = {
        let _ = location.set_extension("exe");
        OpenOptions::new()
          .create(true)
          .write(true)
          .open(&location)?
      };
      let _ = program.copy_to(&mut file)?;
      let _ = location.pop();
    }
  }

  // We need to add this to the PATH for the local user
  #[cfg(target_os = "windows")]
  {
    println!("Adding {} to your %PATH%", location.display());
    let mut location = location.into_os_string();
    location.push(";%PATH%");
    let _ = Command::new("setx").arg("PATH").arg(&location).output()?;
    println!("You'll need to restart your computer for the %PATH% changes to take effect");
  }
  println!("Installation complete");

  Ok(())
}
