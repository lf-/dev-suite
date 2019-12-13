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

        println!(
          "{}{}\n\n{}",
          "Status: ".bold().purple(),
          match ticket.status {
            Status::Open => "Open".bold().green(),
            Status::Closed => "Closed".bold().red(),
          },
          ticket.description
        );
        for (_, name, comment) in ticket.comments.values() {
          println!("{}\n{}\n", name.0.cyan(), comment.0);
        }
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
  let tickets = get_all_ticketsv0()?;

  let open_tickets_path = open_tickets()?;
  let closed_tickets_path = closed_tickets()?;

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
  for (k, v) in &ticket.comments {
    info!("{:?} = {:?}", k, v);
  }
  let open_tickets_path = open_tickets()?;
  let closed_tickets_path = closed_tickets()?;
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
