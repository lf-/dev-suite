use crate::{
  Ticket,
  TicketV0,
};
use anyhow::{
  bail,
  Result,
};
use chrono::prelude::*;
use log::*;
use rand::prelude::*;
use shared::find_root;
use std::{
  convert::TryInto,
  fs,
  path::{
    Path,
    PathBuf,
  },
};
use uuid::{
  v1::{
    Context,
    Timestamp,
  },
  Uuid,
};

pub fn get_all_tickets() -> Result<Vec<Ticket>> {
  let mut tickets = get_open_tickets()?;
  tickets.extend(get_closed_tickets()?);
  Ok(tickets)
}

pub fn get_open_tickets() -> Result<Vec<Ticket>> {
  get_tickets(&open_tickets()?)
}

pub fn get_closed_tickets() -> Result<Vec<Ticket>> {
  get_tickets(&closed_tickets()?)
}

fn get_tickets(path: &Path) -> Result<Vec<Ticket>> {
  let mut out = Vec::new();
  debug!("Looking for ticket.");
  for entry in fs::read_dir(&path)? {
    let entry = entry?;
    let path = entry.path();
    trace!("Looking at entry {}.", path.display());
    if path.is_file() {
      trace!("Entry is a file.");
      match toml::from_slice::<Ticket>(&fs::read(&path)?) {
        Ok(ticket) => out.push(ticket),
        Err(e) => {
          error!("Failed to parse ticket {}", path.canonicalize()?.display());
          error!("Is the file an old ticket format? You might need to run `ticket migrate`.");
          bail!("Underlying error was {}", e);
        }
      }
    }
  }
  out.sort_by(|a, b| a.id.cmp(&b.id));
  Ok(out)
}

pub fn ticket_root() -> Result<PathBuf> {
  Ok(find_root()?.join(".dev-suite").join("ticket"))
}

pub fn closed_tickets() -> Result<PathBuf> {
  Ok(ticket_root()?.join("closed"))
}

pub fn open_tickets() -> Result<PathBuf> {
  Ok(ticket_root()?.join("open"))
}

// Old version ticket code to handle grabbing code
pub fn get_all_ticketsv0() -> Result<Vec<TicketV0>> {
  let mut tickets = get_open_ticketsv0()?;
  tickets.extend(get_closed_ticketsv0()?);
  Ok(tickets)
}
pub fn get_open_ticketsv0() -> Result<Vec<TicketV0>> {
  get_ticketsv0(&open_tickets()?)
}

pub fn get_closed_ticketsv0() -> Result<Vec<TicketV0>> {
  get_ticketsv0(&closed_tickets()?)
}

fn get_ticketsv0(path: &Path) -> Result<Vec<TicketV0>> {
  let mut out = Vec::new();
  debug!("Looking for ticket.");
  for entry in fs::read_dir(&path)? {
    let entry = entry?;
    let path = entry.path();
    trace!("Looking at entry {}.", path.display());
    if path.is_file() {
      trace!("Entry is a file.");
      if let Ok(ticket) = toml::from_slice::<TicketV0>(&fs::read(&path)?) {
        out.push(ticket);
      }
    }
  }
  out.sort_by(|a, b| a.number.cmp(&b.number));
  Ok(out)
}

pub fn uuid_v1() -> Result<Uuid> {
  Ok(Uuid::new_v1(
    Timestamp::from_unix(
      Context::new(random()),
      Utc::now().timestamp().try_into()?,
      0,
    ),
    &[random(), random(), random(), random(), random(), random()],
  )?)
}
