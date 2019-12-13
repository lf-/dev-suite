//! ticket is a cli tool to create, delete, and manage tickets as part of
//! repository, rather than a separate service outside the history of the
//! code.
mod actions;
mod tui;

use actions::*;
use anyhow::{
  bail,
  format_err,
  Result,
};
use colored::*;
use configamajig::*;
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
  collections::BTreeMap,
  env,
  fs,
  process,
  process::Command,
  thread,
  time,
};
use uuid::Uuid;

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
  /// Close a ticket from the command line
  Close { id: Uuid },
  /// Comment on a ticket from the command line
  Comment { id: Uuid, message: String },
}

#[paw::main]
fn main(args: Args) {
  env::var("RUST_LOG")
    .ok()
    .map_or_else(|| env::set_var("RUST_LOG", "info"), drop);
  pretty_env_logger::init();

  if let Some(cmd) = args.cmd {
    if let Err(e) = match cmd {
      Cmd::Init => init(),
      Cmd::New => new(),
      Cmd::Migrate => migrate(),
      Cmd::Show { id } => show(id),
      Cmd::Close { id } => close(id),
      Cmd::Comment { id, message } => comment(id, message),
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
  debug!("Creating open ticket directory.");
  fs::create_dir_all(&open_tickets()?)?;
  debug!("Creating closed ticket directory");
  fs::create_dir_all(&closed_tickets()?)?;
  trace!("Done initializing tickets.");
  Ok(())
}

fn new() -> Result<()> {
  debug!("Getting ticket root.");
  let ticket_root = ticket_root()?;
  trace!("Got ticket root: {}", ticket_root.display());
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
  let _ = fs::File::create(&description)?;
  let _ = Command::new(&env::var("EDITOR").unwrap_or_else(|_| "vi".into()))
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
    id: uuid_v1()?,
    assignees: Vec::new(),
    description: description_contents,
    comments: BTreeMap::new(),
    version: Version::V1,
  };

  save_ticket(t)
}

fn show(id: Uuid) -> Result<()> {
  let mut found = false;
  for ticket in get_all_tickets()? {
    if ticket.id == id {
      println!(
        "{}\n{}{}\n{}{}\n\n{}\n{}",
        format!("{} - {}\n", ticket.id, ticket.title).bold().red(),
        "Status: ".bold().purple(),
        match ticket.status {
          Status::Open => "Open".bold().green(),
          Status::Closed => "Closed".bold().red(),
        },
        "Assignees: ".bold().purple(),
        if ticket.assignees.is_empty() {
          "None".to_owned().blue()
        } else {
          ticket.assignees.join(", ").blue()
        },
        ticket.description,
        ticket.comments.values().fold(
          String::new(),
          |mut acc, (_, name, comment)| {
            acc.push_str(&format!("{}\n{}", name.0.cyan(), comment.0));
            acc
          }
        )
      );
      found = true;
      break;
    }
  }
  if found {
    Ok(())
  } else {
    bail!("No ticket with id {} exists.", id);
  }
}

fn close(id: Uuid) -> Result<()> {
  let mut found = false;
  for mut ticket in get_open_tickets()? {
    if ticket.id == id {
      let path = ticket_path(&ticket)?;
      ticket.status = Status::Closed;
      save_ticket(ticket)?;
      fs::remove_file(path)?;
      found = true;
      break;
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
  let tickets = get_all_ticketsv0()?;

  for t in tickets {
    let ticket = Ticket {
      title: t.title,
      status: t.status,
      id: uuid_v1()?,
      assignees: t.assignee.map_or_else(Vec::new, |a| vec![a]),
      description: t.description,
      comments: BTreeMap::new(),
      version: Version::V1,
    };
    let mut path = ticket_path(&ticket)?;
    let _ = path.pop();
    fs::remove_file(path.join(format!(
      "{}-{}",
      t.number,
      ticket_file_name(&ticket)
    )))?;
    save_ticket(ticket)?;
    // We need to make sure we get different times for each ticket
    // Possible future migrations might not have this issue
    thread::sleep(time::Duration::from_millis(1000));
  }
  Ok(())
}

fn comment(id: Uuid, message: String) -> Result<()> {
  let mut ticket = get_all_tickets()?
    .into_iter()
    .find(|t| t.id == id)
    .ok_or_else(|| {
      format_err!("The uuid '{}' is not associated with any ticket")
    })?;
  let user_config = get_user_config()?;
  let _ = ticket.comments.insert(
    uuid_v1()?,
    (user_config.uuid, Name(user_config.name), Comment(message)),
  );
  save_ticket(ticket)?;
  Ok(())
}
#[derive(Serialize, Deserialize, Debug)]
/// The fundamental type this tool revolves around. The ticket represents
/// everything about an issue or future plan for the code base.
pub struct Ticket {
  title: String,
  status: Status,
  id: Uuid,
  assignees: Vec<String>,
  description: String,
  version: Version,
  #[serde(serialize_with = "toml::ser::tables_last")]
  comments: BTreeMap<Uuid, (Uuid, Name, Comment)>,
}

#[derive(Serialize, Deserialize, Debug)]
/// Enum representing what version of the ticket it is and the assumptions that
/// can be made about it
pub enum Version {
  /// The first version
  V1,
}

#[derive(Serialize, Deserialize, Debug)]
/// Newtype to represent a users Name
pub struct Name(String);

#[derive(Serialize, Deserialize, Debug)]
/// Newtype to represent a Comment
pub struct Comment(String);

#[derive(Serialize, Deserialize, Debug)]
/// Original version of the tickets on disk. This exists for historical reasons
/// but is deprecated and likely to be removed.
pub struct TicketV0 {
  title: String,
  status: Status,
  number: usize,
  assignee: Option<String>,
  description: String,
}

#[derive(Serialize, Deserialize, Debug)]
/// What is the current state of a ticket
pub enum Status {
  /// The ticket has been opened but the issue has not been resolved
  Open,
  /// The ticket has a corresponding fix and has been closed
  Closed,
}
