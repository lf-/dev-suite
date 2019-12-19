use crate::{
  actions::{
    get_closed_tickets,
    get_open_tickets,
  },
  Status,
  Ticket,
};
use anyhow::Result;
use crossterm::{
  cursor::Hide,
  event::{
    self,
    DisableMouseCapture,
    EnableMouseCapture,
    Event as CEvent,
    KeyCode,
  },
  queue,
  terminal::*,
};
use std::{
  collections::BTreeMap,
  io::{
    self,
    Write,
  },
  sync::mpsc,
  thread,
  time::Duration,
};
use tui::{
  backend::CrosstermBackend,
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
struct App<'a> {
  tabs: TabsState<'a>,
  tickets: TicketState,
  should_quit: bool,
}
#[derive(Debug, Clone, Copy)]
pub struct Config {
  pub exit_key: KeyCode,
  pub tick_rate: Duration,
}

impl Default for Config {
  fn default() -> Self {
    Self {
      exit_key: KeyCode::Char('q'),
      tick_rate: Duration::from_millis(250),
    }
  }
}
pub fn run() -> Result<()> {
  // Terminal initialization
  enable_raw_mode()?;
  queue!(io::stdout(), EnterAlternateScreen, EnableMouseCapture, Hide)?;
  let backend = CrosstermBackend::new(io::stdout());
  let mut terminal = Terminal::new(backend)?;

  // App
  let mut app = App {
    tabs: TabsState::new(vec!["Open", "Closed"]),
    tickets: {
      let mut map = BTreeMap::new();
      let _ = map.insert("Open".into(), get_open_tickets()?);
      let _ = map.insert("Closed".into(), get_closed_tickets()?);
      TicketState::new(map)
    },
    should_quit: false,
  };

  terminal.clear()?;
  let (tx, rx) = mpsc::channel();
  let _ = thread::spawn(move || {
    loop {
      // poll for tick rate duration, if no events, sent tick event.
      if event::poll(Duration::from_millis(250)).unwrap() {
        if let CEvent::Key(key) = event::read().unwrap() {
          tx.send(Event::Input(key)).unwrap();
        }
      }

      tx.send(Event::Tick).unwrap();
    }
  });

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

    match rx.recv()? {
      Event::Input(event) => match event.code {
        KeyCode::Char('q') => {
          app.should_quit = true;
        }
        KeyCode::Right => {
          if app.tabs.index == 0 {
            app.tickets.status = Status::Closed;
            app.tickets.index = 0;
          }
          app.tabs.next();
        }
        KeyCode::Left => {
          if app.tabs.index != 0 {
            app.tickets.status = Status::Open;
            app.tickets.index = 0;
          }
          app.tabs.previous();
        }
        KeyCode::Up => app.tickets.previous(),
        KeyCode::Down => app.tickets.next(),
        _ => {}
      },
      Event::Tick => continue,
    }
    if app.should_quit {
      break;
    }
  }
  queue!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
  disable_raw_mode()?;
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
