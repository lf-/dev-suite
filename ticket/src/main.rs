mod actions;
mod tui;

use actions::*;
use anyhow::{
  bail,
  Result,
};
use colored::*;
use log::*;
use rustyline::{
  error::ReadlineError,
  Editor,
};
use serde::{
  Deserialize,
  Serialize,
};
use std::{
  env,
  fs,
  process,
  process::Command,
};

#[derive(structopt::StructOpt)]
struct Args {
  #[structopt(subcommand)]
  cmd: Option<Cmd>,
}

#[derive(structopt::StructOpt)]
enum Cmd {
  /// Initialize the repo to use ticket
  Init,
  New,
  Show {
    id: usize,
  },
  Close {
    id: usize,
  },
}

#[paw::main]
fn main(args: Args) {
  env::var("RUST_LOG").map(drop).unwrap_or_else(|_| {
    env::set_var("RUST_LOG", "info");
  });
  pretty_env_logger::init();

  if let Some(cmd) = args.cmd {
    if let Err(e) = match cmd {
      Cmd::Init => init(),
      Cmd::New => new(),
      Cmd::Show { id } => show(id),
      Cmd::Close { id } => close(id),
    } {
      error!("{}", e);
      std::process::exit(1);
    }
  } else if let Err(e) = tui::run() {
    error!("{}", e);
    std::process::exit(1);
  }
}

fn init() -> Result<()> {
  let root = ticket_root()?;
  debug!("Creating ticket directory at {}.", root.display());
  debug!("Creating open directory.");
  fs::create_dir_all(&root.join("open"))?;
  debug!("Creating closed directory");
  fs::create_dir_all(&root.join("closed"))?;
  trace!("Done initializing tickets.");
  info!("Created ticket directory at {}.", root.display());
  Ok(())
}

fn new() -> Result<()> {
  debug!("Getting ticket root.");
  let ticket_root = ticket_root()?;
  trace!("Got ticket root: {}", ticket_root.display());
  let open = ticket_root.join("open");
  let closed = ticket_root.join("closed");
  let description = ticket_root.join("description");
  let mut ticket_num = 1;

  // Fast enough for now but maybe not in the future
  debug!("Getting number of tickets total.");
  for entry in fs::read_dir(&open)?.chain(fs::read_dir(&closed)?) {
    let entry = entry?;
    let path = entry.path();
    if path.is_file() {
      ticket_num += 1;
    }
  }
  debug!("Ticket Total: {}", ticket_num - 1);
  debug!("Next Ticket ID: {}", ticket_num);

  let mut rl = Editor::<()>::new();
  let title = match rl.readline("Title: ") {
    Ok(line) => {
      if line.is_empty() {
        bail!("Title may not be empty");
      }
      line
    }
    Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
      debug!("Exiting due to Ctrl-C or Ctrl-D.");
      process::exit(0);
    }
    Err(e) => return Err(e.into()),
  };

  debug!("Opening up editor.");
  trace!(
    "Create buffer file for the description at {}.",
    description.display()
  );
  fs::File::create(&description)?;
  Command::new(&env::var("EDITOR").unwrap_or_else(|_| "vi".into()))
    .arg(&description)
    .spawn()?
    .wait()?;
  trace!("Read the file into memory.");
  let description_contents = fs::read_to_string(&description)?;
  trace!("Removing the file.");
  fs::remove_file(&description)?;

  debug!("Creating ticket in memory.");
  let t = Ticket {
    title,
    status: Status::Open,
    number: ticket_num,
    assignee: None,
    description: description_contents,
  };

  debug!("Converting ticket to toml and writing to disk.");
  fs::write(
    open.join(&format!(
      "{}-{}.toml",
      ticket_num,
      t.title
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join("-")
    )),
    toml::to_string_pretty(&t)?,
  )?;
  trace!("Finished writing data to disk.");

  Ok(())
}

fn show(id: usize) -> Result<()> {
  debug!("Getting ticket root.");
  let ticket_root = ticket_root()?;
  trace!("Ticket root at {}.", ticket_root.display());
  let open = ticket_root.join("open");
  let closed = ticket_root.join("closed");
  let mut found = false;

  // Fast enough for now but maybe not in the future
  debug!("Looking for ticket.");
  for entry in fs::read_dir(&open)?.chain(fs::read_dir(&closed)?) {
    let entry = entry?;
    let path = entry.path();
    trace!("Looking at entry {}.", path.display());
    if path.is_file() {
      if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
        trace!("Entry is a file.");
        if file_name.starts_with(&id.to_string()) {
          trace!("This is the expected entry.");
          let ticket = toml::from_slice::<Ticket>(&fs::read(&path)?)?;
          println!(
            "{}",
            format!("{} - {}\n", ticket.number, ticket.title)
              .bold()
              .red()
          );
          if let Some(a) = ticket.assignee {
            println!("{}{}", "Assignee: ".bold().purple(), a);
          }

          print!(
            "{}{}\n\n{}",
            "Status: ".bold().purple(),
            match ticket.status {
              Status::Open => "Open".bold().green(),
              Status::Closed => "Closed".bold().red(),
            },
            ticket.description
          );
          found = true;
          break;
        }
      }
    }
  }
  if found {
    Ok(())
  } else {
    bail!("No ticket with id {} exists.", id);
  }
}

fn close(id: usize) -> Result<()> {
  debug!("Getting ticket root.");
  let ticket_root = ticket_root()?;
  trace!("Ticket root at {}.", ticket_root.display());
  let open = ticket_root.join("open");
  let closed = ticket_root.join("closed");
  let mut found = false;
  // Fast enough for now but maybe not in the future
  debug!("Looking for open ticket with id {}", id);
  for entry in fs::read_dir(&open)? {
    let entry = entry?;
    let path = entry.path();
    trace!("Looking at entry {}.", path.display());
    if path.is_file() {
      if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
        if file_name.starts_with(&id.to_string()) {
          trace!("The ticket is open and exists.");
          debug!("Reading in the ticket from disk and setting it to closed.");
          let mut ticket = toml::from_slice::<Ticket>(&fs::read(&path)?)?;
          ticket.status = Status::Closed;
          debug!("Writing ticket to disk in the closed directory.");
          fs::write(closed.join(file_name), toml::to_string_pretty(&ticket)?)?;
          debug!("Removing old ticket.");
          fs::remove_file(&path)?;
          trace!("Removed the old ticket.");
          found = true;
          break;
        }
      }
    }
  }
  if found {
    Ok(())
  } else {
    bail!("No ticket with id {} exists.", id);
  }
}

#[derive(Serialize, Deserialize)]
pub struct Ticket {
  title: String,
  status: Status,
  number: usize,
  assignee: Option<String>,
  description: String,
}

#[derive(Serialize, Deserialize)]
pub enum Status {
  Open,
  Closed,
}
