use crate::{
  actions::{
    get_closed_tickets,
    get_open_tickets,
  },
  Status,
  Ticket,
};
use anyhow::Result;
use std::{
  collections::BTreeMap,
  io,
  sync::mpsc,
  thread,
  time::Duration,
};
use termion::{
  event::Key,
  input::{
    MouseTerminal,
    TermRead,
  },
  raw::IntoRawMode,
  screen::AlternateScreen,
};
use tui::{
  backend::TermionBackend,
  layout::{
    Alignment,
    Constraint,
    Direction,
    Layout,
  },
  style::{
    Color,
    Modifier,
    Style,
  },
  widgets::{
    Block,
    Borders,
    Paragraph,
    Row,
    Table,
    Tabs,
    Text,
    Widget,
  },
  Terminal,
};

pub struct TabsState<'a> {
  pub titles: Vec<&'a str>,
  pub index: usize,
}

impl<'a> TabsState<'a> {
  pub fn new(titles: Vec<&'a str>) -> TabsState {
    TabsState { titles, index: 0 }
  }

  pub fn next(&mut self) {
    self.index = (self.index + 1) % self.titles.len();
  }

  pub fn previous(&mut self) {
    if self.index > 0 {
      self.index -= 1;
    } else {
      self.index = self.titles.len() - 1;
    }
  }
}
pub enum Event<I> {
  Input(I),
  Tick,
}

pub struct TicketState {
  pub tickets: BTreeMap<String, Vec<Ticket>>,
  pub index: usize,
  pub status: Status,
}

impl TicketState {
  pub fn new(tickets: BTreeMap<String, Vec<Ticket>>) -> Self {
    Self {
      tickets,
      index: 0,
      status: Status::Open,
    }
  }

  fn len(&self) -> usize {
    match self.status {
      Status::Open => self.tickets.get("Open").unwrap().len(),
      Status::Closed => self.tickets.get("Closed").unwrap().len(),
    }
  }

  pub fn next(&mut self) {
    self.index = (self.index + 1) % self.len();
  }

  pub fn previous(&mut self) {
    if self.index > 0 {
      self.index -= 1;
    } else {
      self.index = self.len() - 1;
    }
  }
}
/// A small event handler that wrap termion input and tick events. Each event
/// type is handled in its own thread and returned to a common `Receiver`
#[allow(dead_code)]
pub struct Events {
  rx: mpsc::Receiver<Event<Key>>,
  input_handle: thread::JoinHandle<()>,
  tick_handle: thread::JoinHandle<()>,
}

struct App<'a> {
  tabs: TabsState<'a>,
  tickets: TicketState,
}
#[derive(Debug, Clone, Copy)]
pub struct Config {
  pub exit_key: Key,
  pub tick_rate: Duration,
}

impl Default for Config {
  fn default() -> Self {
    Self {
      exit_key: Key::Char('q'),
      tick_rate: Duration::from_millis(250),
    }
  }
}

impl Events {
  pub fn new() -> Self {
    Self::with_config(Config::default())
  }

  pub fn with_config(config: Config) -> Self {
    let (tx, rx) = mpsc::channel();
    let input_handle = {
      let tx = tx.clone();
      thread::spawn(move || {
        let stdin = io::stdin();
        for evt in stdin.keys() {
          if let Ok(key) = evt {
            if tx.send(Event::Input(key)).is_err() {
              return;
            }
            if key == config.exit_key {
              return;
            }
          }
        }
      })
    };
    let tick_handle = {
      let tx = tx.clone();
      thread::spawn(move || {
        let tx = tx.clone();
        loop {
          tx.send(Event::Tick).unwrap();
          thread::sleep(config.tick_rate);
        }
      })
    };
    Self {
      rx,
      input_handle,
      tick_handle,
    }
  }

  pub fn next(&self) -> Result<Event<Key>, mpsc::RecvError> {
    self.rx.recv()
  }
}
pub fn run() -> Result<()> {
  // Terminal initialization
  let stdout = io::stdout().into_raw_mode()?;
  let stdout = MouseTerminal::from(stdout);
  let stdout = AlternateScreen::from(stdout);
  let backend = TermionBackend::new(stdout);
  let mut terminal = Terminal::new(backend)?;
  terminal.hide_cursor()?;

  let events = Events::new();

  // App
  let mut app = App {
    tabs: TabsState::new(vec!["Open", "Closed"]),
    tickets: {
      let mut map = BTreeMap::new();
      let _ = map.insert("Open".into(), get_open_tickets()?);
      let _ = map.insert("Closed".into(), get_closed_tickets()?);
      TicketState::new(map)
    },
  };

  // Main loop
  loop {
    terminal.draw(|mut f| {
      let size = f.size();
      let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(size);
      let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .vertical_margin(3)
        .constraints(
          [Constraint::Percentage(30), Constraint::Percentage(70)].as_ref(),
        )
        .split(size);

      Tabs::default()
        .block(Block::default().borders(Borders::ALL).title("Status"))
        .titles(&app.tabs.titles)
        .select(app.tabs.index)
        .style(Style::default().fg(Color::Cyan))
        .highlight_style(Style::default().fg(Color::Yellow))
        .render(&mut f, vertical[0]);

      match app.tabs.index {
        0 => {
          app.table("Open").render(&mut f, horizontal[0]);

          Paragraph::new(app.description("Open").iter())
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Left)
            .wrap(true)
            .render(&mut f, horizontal[1]);
        }
        1 => {
          app.table("Closed").render(&mut f, horizontal[0]);

          Paragraph::new(app.description("Closed").iter())
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Left)
            .wrap(true)
            .render(&mut f, horizontal[1]);
        }
        _ => {}
      }
    })?;

    match events.next()? {
      Event::Input(input) => match input {
        Key::Char('q') => {
          break;
        }
        Key::Right => {
          if app.tabs.index == 0 {
            app.tickets.status = Status::Closed;
            app.tickets.index = 0;
          }
          app.tabs.next();
        }
        Key::Left => {
          if app.tabs.index != 0 {
            app.tickets.status = Status::Open;
            app.tickets.index = 0;
          }
          app.tabs.previous();
        }
        Key::Up => app.tickets.previous(),
        Key::Down => app.tickets.next(),
        _ => {}
      },
      Event::Tick => continue,
    }
  }
  Ok(())
}

impl<'a> App<'a> {
  fn table(&self, tab: &'a str) -> impl Widget + '_ {
    Table::new(
      ["Id", "Title"].iter(),
      self
        .tickets
        .tickets
        .get(tab)
        .unwrap()
        .iter()
        .enumerate()
        .map(move |(idx, i)| {
          let data = vec![i.id.to_string(), i.title.to_string()].into_iter();
          let normal_style = Style::default().fg(Color::Yellow);
          let selected_style =
            Style::default().fg(Color::White).modifier(Modifier::BOLD);
          if idx == self.tickets.index {
            Row::StyledData(data, selected_style)
          } else {
            Row::StyledData(data, normal_style)
          }
        }),
    )
    .block(Block::default().title(tab).borders(Borders::ALL))
    .header_style(Style::default().fg(Color::Yellow))
    .widths(&[Constraint::Percentage(30), Constraint::Percentage(70)])
    .style(Style::default().fg(Color::White))
    .column_spacing(1)
  }

  fn description(&self, tab: &'a str) -> Vec<Text> {
    let mut description = vec![];
    for (idx, i) in self.tickets.tickets.get(tab).unwrap().iter().enumerate() {
      if idx == self.tickets.index {
        description = {
          let header = Style::default().fg(Color::Red).modifier(Modifier::BOLD);
          let mut desc = vec![
            Text::styled("Description\n-------------\n", header),
            Text::raw(i.description.to_owned()),
          ];
          let name_style =
            Style::default().fg(Color::Cyan).modifier(Modifier::BOLD);
          if i.comments.is_empty() {
            desc.push(Text::styled("\nComments\n--------\n", header));
            for (_, name, comment) in i.comments.values() {
              desc.push(Text::styled(format!("\n{}\n", name.0), name_style));
              desc.push(Text::raw(format!("{}\n", comment.0)));
            }
          }
          desc
        };
        break;
      }
    }

    description
  }
}
