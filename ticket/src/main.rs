use anyhow::{
  bail,
  Result,
};
use colored::*;
use rustyline::{
  error::ReadlineError,
  Editor,
};
use serde::{
  Deserialize,
  Serialize,
};
use shared::find_root;
use std::{
  env,
  fs,
  path::PathBuf,
  process,
  process::Command,
};

#[derive(structopt::StructOpt)]
enum Args {
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
  if let Err(e) = match args {
    Args::Init => init(),
    Args::New => new(),
    Args::Show { id } => show(id),
    Args::Close { id } => close(id),
  } {
    eprintln!("{}", e);
    std::process::exit(1);
  }
}

fn init() -> Result<()> {
  let root = find_root()?.join(".dev-suite").join("ticket");
  fs::create_dir_all(&root.join("open"))?;
  fs::create_dir_all(&root.join("closed"))?;
  Ok(())
}

fn new() -> Result<()> {
  let ticket_root = ticket_root()?;
  let open = ticket_root.join("open");
  let closed = ticket_root.join("closed");
  let description = ticket_root.join("description");
  let mut ticket_num = 1;

  // Fast enough for now but maybe not in the future
  for entry in fs::read_dir(&open)?.chain(fs::read_dir(&closed)?) {
    let entry = entry?;
    let path = entry.path();
    if path.is_file() {
      ticket_num += 1;
    }
  }

  let mut rl = Editor::<()>::new();
  let title = match rl.readline("Title: ") {
    Ok(line) => {
      if line.is_empty() {
        bail!("Title may not be empty");
      }
      line
    }
    Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
      process::exit(0);
    }
    Err(e) => return Err(e.into()),
  };

  fs::File::create(&description)?;
  Command::new(&env::var("EDITOR").unwrap_or_else(|_| "vi".into()))
    .arg(&description)
    .spawn()?
    .wait()?;
  let description_contents = fs::read_to_string(&description)?;
  fs::remove_file(&description)?;

  let t = Ticket {
    title,
    status: Status::Open,
    number: ticket_num,
    assignee: None,
    description: description_contents,
  };

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

  Ok(())
}

fn ticket_root() -> Result<PathBuf> {
  Ok(find_root()?.join(".dev-suite").join("ticket"))
}

fn show(id: usize) -> Result<()> {
  let ticket_root = ticket_root()?;
  let open = ticket_root.join("open");
  let closed = ticket_root.join("closed");
  let mut found = false;

  // Fast enough for now but maybe not in the future
  for entry in fs::read_dir(&open)?.chain(fs::read_dir(&closed)?) {
    let entry = entry?;
    let path = entry.path();
    if path.is_file() {
      if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
        if file_name.starts_with(&id.to_string()) {
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
    bail!("No ticket with id {} exists", id);
  }
}

fn close(id: usize) -> Result<()> {
  let ticket_root = ticket_root()?;
  let open = ticket_root.join("open");
  let closed = ticket_root.join("closed");
  let mut found = false;
  // Fast enough for now but maybe not in the future
  for entry in fs::read_dir(&open)? {
    let entry = entry?;
    let path = entry.path();
    if path.is_file() {
      if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
        if file_name.starts_with(&id.to_string()) {
          let mut ticket = toml::from_slice::<Ticket>(&fs::read(&path)?)?;
          ticket.status = Status::Closed;
          fs::write(closed.join(file_name), toml::to_string_pretty(&ticket)?)?;
          fs::remove_file(&path)?;
          found = true;
          break;
        }
      }
    }
  }
  if found {
    Ok(())
  } else {
    bail!("No ticket with id {} exists", id);
  }
}

#[derive(Serialize, Deserialize)]
struct Ticket {
  title: String,
  status: Status,
  number: usize,
  assignee: Option<String>,
  description: String,
}

#[derive(Serialize, Deserialize)]
enum Status {
  Open,
  Closed,
}
