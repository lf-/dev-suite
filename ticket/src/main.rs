mod actions;
mod tui;

use actions::*;
use anyhow::{
  bail,
  Result,
};
use chrono::prelude::*;
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
  thread,
  time,
};
use uuid::{
  v1::{
    Context,
    Timestamp,
  },
  Uuid,
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
  /// Update tickets to newer formats
  Migrate,
  /// Create a new ticket
  New,
  /// Show a ticket on the command line
  Show { id: Uuid },
  /// Close a ticket on the command line
  Close { id: Uuid },
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
      Cmd::Migrate => migrate(),
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
  let open = open_tickets()?;
  let description = ticket_root.join("description");

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
    id: Uuid::new_v1(
      Timestamp::from_unix(Context::new(1), Utc::now().timestamp() as u64, 0),
      &[0, 5, 2, 4, 9, 3],
    )?,
    assignees: Vec::new(),
    description: description_contents,
    comments: Vec::new(),
    version: Version::V1,
  };

  debug!("Converting ticket to toml and writing to disk.");
  fs::write(
    open.join(&format!(
      "{}.toml",
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

fn show(id: Uuid) -> Result<()> {
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
      let ticket = toml::from_slice::<Ticket>(&fs::read(&path)?)?;
      if ticket.id == id {
        trace!("This is the expected entry.");
        println!(
          "{}",
          format!("{} - {}\n", ticket.id, ticket.title).bold().red()
        );
        if !ticket.assignees.is_empty() {
          println!(
            "{}{}",
            "Assignees: ".bold().purple(),
            ticket.assignees.join(", ")
          );
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
  if found {
    Ok(())
  } else {
    bail!("No ticket with id {} exists.", id);
  }
}

fn close(id: Uuid) -> Result<()> {
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
      let mut ticket = toml::from_slice::<Ticket>(&fs::read(&path)?)?;
      if ticket.id == id {
        debug!("Ticket found setting it to closed.");
        ticket.status = Status::Closed;
        trace!("Writing ticket to disk in the closed directory.");
        fs::write(
          closed.join(path.file_name().expect("Path should have a file name")),
          toml::to_string_pretty(&ticket)?,
        )?;
        debug!("Removing old ticket.");
        fs::remove_file(&path)?;
        trace!("Removed the old ticket.");
        found = true;
        break;
      }
    }
  }
  if found {
    Ok(())
  } else {
    bail!("No ticket with id {} exists.", id);
  }
}

/// Upgrade from V0 to V1 of the ticket
fn migrate() -> Result<()> {
  let ctx = Context::new(1);
  let tickets = get_all_ticketsv0()?;

  let open_tickets_path = open_tickets()?;
  let closed_tickets_path = closed_tickets()?;

  for t in tickets.into_iter() {
    let ticket = Ticket {
      title: t.title,
      status: t.status,
      id: Uuid::new_v1(
        Timestamp::from_unix(&ctx, Utc::now().timestamp() as u64, 0),
        &[0, 5, 2, 4, 9, 3],
      )?,
      assignees: t.assignee.map(|a| vec![a]).unwrap_or_else(Vec::new),
      description: t.description,
      comments: Vec::new(),
      version: Version::V1,
    };

    let path = match ticket.status {
      Status::Open => &open_tickets_path,
      Status::Closed => &closed_tickets_path,
    };

    let mut name = ticket
      .title
      .split_whitespace()
      .collect::<Vec<&str>>()
      .join("-");
    name.push_str(".toml");
    name = name.to_lowercase();
    fs::write(path.join(&name), toml::to_string_pretty(&ticket)?)?;
    fs::remove_file(path.join(format!("{}-{}", t.number, name)))?;
    // We need to make sure we get different times for each ticket
    // Possible future migrations might not have this issue
    thread::sleep(time::Duration::from_millis(1000));
  }
  Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct Ticket {
  title: String,
  status: Status,
  id: Uuid,
  assignees: Vec<String>,
  description: String,
  comments: Vec<(User, String)>,
  version: Version,
}

#[derive(Serialize, Deserialize)]
pub enum Version {
  V1,
}

#[derive(Serialize, Deserialize)]
pub struct User(String);

#[derive(Serialize, Deserialize)]
pub struct TicketV0 {
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
