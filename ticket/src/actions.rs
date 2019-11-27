use crate::Ticket;
use anyhow::Result;
use log::*;
use shared::find_root;
use std::{
  fs,
  path::PathBuf,
};

pub fn get_open_tickets() -> Result<Vec<Ticket>> {
  get_tickets(ticket_root()?.join("open"))
}

pub fn get_closed_tickets() -> Result<Vec<Ticket>> {
  get_tickets(ticket_root()?.join("closed"))
}

fn get_tickets(path: PathBuf) -> Result<Vec<Ticket>> {
  let mut out = Vec::new();
  debug!("Looking for ticket.");
  for entry in fs::read_dir(&path)? {
    let entry = entry?;
    let path = entry.path();
    trace!("Looking at entry {}.", path.display());
    if path.is_file() {
      trace!("Entry is a file.");
      out.push(toml::from_slice::<Ticket>(&fs::read(&path)?)?);
    }
  }
  out.sort_by(|a, b| a.number.cmp(&b.number));
  Ok(out)
}

pub fn ticket_root() -> Result<PathBuf> {
  Ok(find_root()?.join(".dev-suite").join("ticket"))
}
