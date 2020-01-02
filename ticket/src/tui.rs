use crate::{
  actions::{
    get_closed_tickets,
    get_open_tickets,
    save_ticket,
    uuid_v1,
  },
  Comment,
  Name,
  Status,
  Ticket,
};
use anyhow::Result;
use configamajig::{
  get_user_config,
  UserConfig,
};
use crossterm::{
  event::{
    self,
    DisableMouseCapture,
    EnableMouseCapture,
    Event as CEvent,
    KeyCode,
    KeyEvent,
  },
  queue,
  terminal::*,
};
use std::{
  collections::BTreeMap,
  io::{
    self,
    BufWriter,
    Write,
  },
  sync::mpsc::{
    self,
    Receiver,
    Sender,
  },
  thread,
  time::Duration,
};
use tui::{
  backend::{
    Backend,
    CrosstermBackend,
  },
  layout::{
    Alignment,
    Constraint,
    Direction,
    Layout,
    Rect,
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
  Frame,
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
    self.index = (self.index + 1) % self.titles.len()
  }

  pub fn previous(&mut self) {
    if self.index > 0 {
      self.index = (self.index - 1) % self.titles.len()
    }
  }
}
pub enum Event<I> {
  Input(I),
  Tick,
}

pub struct TicketState {
  pub tickets: BTreeMap<String, Vec<(Ticket, String)>>,
  pub index: usize,
  pub status: Status,
}

impl TicketState {
  pub fn new(tickets: BTreeMap<String, Vec<(Ticket, String)>>) -> Self {
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
    self.index = (self.index + 1) % self.len()
  }

  pub fn previous(&mut self) {
    if self.index > 0 {
      self.index = (self.index - 1) % self.len()
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

#[allow(clippy::too_many_lines)]
pub fn run() -> Result<()> {
  let stdout = io::stdout();
  let mut lock = BufWriter::new(stdout.lock());
  // Terminal initialization
  enable_raw_mode()?;
  queue!(lock, EnterAlternateScreen, EnableMouseCapture)?;
  let mut terminal = Terminal::new(CrosstermBackend::new(lock))?;
  terminal.backend_mut().hide_cursor()?;
  terminal.clear()?;

  // App
  let mut app = App {
    tabs: TabsState::new(vec!["Open", "Closed"]),
    tickets: {
      let mut map = BTreeMap::new();
      let _ = map.insert(
        "Open".into(),
        get_open_tickets()?
          .into_iter()
          .map(|i| (i, String::new()))
          .collect(),
      );
      let _ = map.insert(
        "Closed".into(),
        get_closed_tickets()?
          .into_iter()
          .map(|i| (i, String::new()))
          .collect(),
      );
      TicketState::new(map)
    },
    should_quit: false,
  };

  // Spawn event sender thread
  let (tx, rx) = mpsc::channel();
  let (tx_close, rx_close) = mpsc::channel();
  let _ = thread::spawn(move || -> Result<()> {
    loop {
      // poll for tick rate duration, if no events, sent tick event.
      if event::poll(Duration::from_millis(250))? {
        if let CEvent::Key(key) = event::read()? {
          tx.send(Event::Input(key))?;
        }
      }

      if rx_close.try_recv().unwrap_or(false) {
        break;
      }

      tx.send(Event::Tick)?;
    }
    Ok(())
  });

  // Cached Values
  let user_config = get_user_config()?;

  // Main drawing and event receiving loop
  loop {
    let status = match app.tickets.status {
      Status::Open => "Open",
      Status::Closed => "Closed",
    };

    terminal.draw(|mut f| {
      let size = f.size();
      let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
          [
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
            Constraint::Length(3),
          ]
          .as_ref(),
        )
        .split(size);
      let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .vertical_margin(3)
        .constraints(
          [Constraint::Percentage(30), Constraint::Percentage(70)].as_ref(),
        )
        .split(Rect {
          x: size.x,
          y: size.y,
          width: size.width,
          height: size.height - 3,
        });
      app.tabs(&mut f, vertical[0]);
      app.table(status, &mut f, horizontal[0]);
      app.description(status, &mut f, horizontal[1]);
      app.comment(status, &mut f, vertical[2]);
      App::instructions(&mut f, vertical[3]);
    })?;

    handle_event(&rx, &tx_close, &mut app, &user_config, &status)?;

    if app.should_quit {
      let open = app.tickets.tickets["Open"].iter();
      let closed = app.tickets.tickets["Closed"].iter();
      for t in open.chain(closed) {
        save_ticket(&t.0)?;
      }
      break;
    }
  }

  // Clean up terminal
  queue!(
    io::stdout().lock(),
    LeaveAlternateScreen,
    DisableMouseCapture
  )?;
  terminal.backend_mut().show_cursor()?;
  disable_raw_mode()?;

  Ok(())
}

fn handle_event(
  rx: &Receiver<Event<KeyEvent>>,
  tx: &Sender<bool>,
  app: &mut App,
  user_config: &UserConfig,
  status: &str,
) -> Result<()> {
  match rx.recv()? {
    Event::Input(event) => match event.code {
      KeyCode::Esc => {
        app.should_quit = true;
        tx.send(true)?;
      }
      KeyCode::Right => {
        if app.tabs.index == 0 {
          app.tickets.status = Status::Closed;
          app.tickets.index = 0;
        }
        app.tabs.next();
      }
      KeyCode::Left => {
        if app.tabs.index > 0 {
          app.tickets.status = Status::Open;
          app.tickets.index = 0;
        }
        app.tabs.previous();
      }
      KeyCode::Up => app.tickets.previous(),
      KeyCode::Down => app.tickets.next(),
      KeyCode::Backspace => {
        let _ = app.tickets.tickets.get_mut(status).unwrap()[app.tickets.index]
          .1
          .pop();
      }
      KeyCode::Char(c) => {
        app.tickets.tickets.get_mut(status).unwrap()[app.tickets.index]
          .1
          .push(c);
      }
      KeyCode::Enter => {
        let ticket =
          &mut app.tickets.tickets.get_mut(status).unwrap()[app.tickets.index];
        if !ticket.1.is_empty() {
          let _ = ticket.0.comments.insert(
            uuid_v1()?,
            (
              user_config.uuid,
              Name(user_config.name.clone()),
              Comment(ticket.1.clone()),
            ),
          );
          ticket.1.clear();
        }
      }
      _ => {}
    },
    Event::Tick => (),
  }
  Ok(())
}

impl<'a> App<'a> {
  #[inline]
  fn table(&self, tab: &'a str, f: &mut Frame<impl Backend>, rect: Rect) {
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
          let data =
            vec![i.0.id.to_string(), i.0.title.to_string()].into_iter();
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
    .render(f, rect)
  }

  #[inline]
  fn description(&self, tab: &'a str, f: &mut Frame<impl Backend>, rect: Rect) {
    let mut description = vec![];
    for (idx, i) in self.tickets.tickets.get(tab).unwrap().iter().enumerate() {
      if idx == self.tickets.index {
        description = {
          let header = Style::default().fg(Color::Red).modifier(Modifier::BOLD);
          let mut desc = vec![
            Text::styled("Description\n-------------\n", header),
            Text::raw(i.0.description.to_owned()),
          ];
          let name_style =
            Style::default().fg(Color::Cyan).modifier(Modifier::BOLD);
          if i.0.assignees.is_empty() {
            desc.push(Text::styled("\nAssignees\n---------\n", header));
          } else {
            desc.push(Text::styled("\nAssignees\n---------\n", header));
            if i.0.assignees.len() == 1 {
              let (_, name) = &i.0.assignees[0];
              desc.push(Text::styled(name.0.clone(), name_style));
            } else {
              for (idx, (_, name)) in i.0.assignees.iter().enumerate() {
                if idx < i.0.assignees.len() - 1 {
                  desc.push(Text::styled(format!("{}, ", name.0), name_style));
                } else {
                  desc.push(Text::styled(name.0.clone(), name_style));
                }
              }
            }
          }

          if i.0.comments.is_empty() {
            desc.push(Text::styled("\nComments\n--------\n", header));
          } else {
            desc.push(Text::styled("\nComments\n--------\n", header));
            for (_, name, comment) in i.0.comments.values() {
              desc.push(Text::styled(format!("{}\n", name.0), name_style));
              desc.push(Text::raw(format!("{}\n\n", comment.0)));
            }
          }
          desc
        };
        break;
      }
    }

    Paragraph::new(description.iter())
      .block(Block::default().borders(Borders::ALL))
      .alignment(Alignment::Left)
      .wrap(true)
      .render(f, rect);
  }

  #[inline]
  fn comment(&self, tab: &'a str, f: &mut Frame<impl Backend>, rect: Rect) {
    let (_, s) = &self.tickets.tickets.get(tab).unwrap()[self.tickets.index];
    let mut text = String::from("> ");
    text.push_str(&s);

    Paragraph::new([Text::raw(text)].iter())
      .block(Block::default().borders(Borders::ALL).title("Comment"))
      .alignment(Alignment::Left)
      .wrap(true)
      .render(f, rect);
  }

  #[inline]
  fn tabs(&self, f: &mut Frame<impl Backend>, rect: Rect) {
    Tabs::default()
      .block(Block::default().borders(Borders::ALL).title("Status"))
      .titles(&self.tabs.titles)
      .select(self.tabs.index)
      .style(Style::default().fg(Color::Cyan))
      .highlight_style(Style::default().fg(Color::Yellow))
      .render(f, rect);
  }

  #[inline]
  fn instructions(f: &mut Frame<impl Backend>, rect: Rect) {
    let blue = Style::default().fg(Color::Blue).modifier(Modifier::BOLD);
    Paragraph::new(
      [
        Text::Styled("[ESC] ".into(), blue),
        Text::Raw("- Exit ".into()),
        Text::Styled("[Enter] ".into(), blue),
        Text::Raw("- Comment ".into()),
        Text::Styled("[Char] ".into(), blue),
        Text::Raw("- Write a comment ".into()),
        Text::Styled("[Backspace] ".into(), blue),
        Text::Raw("- Delete a character".into()),
      ]
      .iter(),
    )
    .block(Block::default().borders(Borders::ALL).title("Instructions"))
    .alignment(Alignment::Left)
    .wrap(true)
    .render(f, rect);
  }
}
