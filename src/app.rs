use std::path::{Path, PathBuf};

use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    prelude::{Backend, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style, Stylize},
    widgets::{Block, Borders, Paragraph, Row, Table, TableState},
    Frame, Terminal,
};
use tui_input::{Input, InputRequest, StateChanged, backend::crossterm::EventHandler};

use crate::{
    db::{Db, Entry},
    search::Bill,
};

pub enum CurrentScreen {
    Main,
    Help,
    Search,
}

pub struct App {
    db: Db,
    db_path: PathBuf,
    entries: Vec<Entry>,
    state: TableState,
    screen: CurrentScreen,
    search_input: Input,
}

impl App {
    pub fn new(db: Db, db_path: &Path) -> Self {
        Self {
            db,
            db_path: db_path.into(),
            entries: Vec::new(),
            state: TableState::default(),
            screen: CurrentScreen::Main,
            search_input: Input::default(),
        }
    }

    pub fn run<B>(&mut self, terminal: &mut Terminal<B>) -> Result<(), Box<dyn std::error::Error>>
    where
        B: Backend,
    {
        self.state.select(Some(0));

        let mut entries: Vec<Entry> = self.db.bills.values().cloned().collect();
        entries.sort_by_key(|e| e.last_checked);
        entries.reverse();

        self.entries = entries;

        loop {
            terminal.draw(|f| ui(f, self))?;

            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Release {
                    continue;
                }

                match self.screen {
                    CurrentScreen::Main => match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('?') => self.screen = CurrentScreen::Help,
                        KeyCode::Char('s') => self.screen = CurrentScreen::Search,
                        KeyCode::Enter => {
                            open::that(self.get_bill().url.clone())?;
                        }
                        KeyCode::Down => self.next(),
                        KeyCode::Up => self.prev(),
                        _ => (),
                    },
                    CurrentScreen::Help => match key.code {
                        KeyCode::Esc => self.screen = CurrentScreen::Main,
                        _ => (),
                    },
                    CurrentScreen::Search => match key.code {
                        KeyCode::Esc => self.screen = CurrentScreen::Main,
                        KeyCode::Enter => {
                            // get_search
                            self.db.saved_searches.push(self.search_input.value().into());
                            self.search_input.reset();
                        }
                        _ => {
                            self.search_input.handle_event(&Event::Key(key));
                        }
                    },
                }
            }
        }

        self.db.write(&self.db_path)?;

        Ok(())
    }

    fn get_bill(&self) -> &Bill {
        let key = self.state.selected().unwrap();
        &self.entries.get(key).unwrap().bill
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.db.bills.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn prev(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.db.bills.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    match app.screen {
        CurrentScreen::Main => {
            let rows = app
                .entries
                .iter()
                .map(|e| Row::new(vec![e.bill.title.clone(), e.bill.last_action_date.clone()]));

            let table = Table::new(
                rows,
                [Constraint::Percentage(50), Constraint::Percentage(10)],
            )
            .block(Block::default().title("billt").borders(Borders::ALL))
            .header(Row::new(vec!["Title", "Last action"]).bold())
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol("> ");

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(90), Constraint::Percentage(10)])
                .split(f.size());

            f.render_stateful_widget(table, chunks[0], &mut app.state);
            f.render_widget(
                Paragraph::new("[up/down] to navigate, [s] to search, [?] for help, [q] to quit.")
                    .block(Block::default().borders(Borders::ALL)),
                chunks[1],
            );
        }
        CurrentScreen::Help => {
            f.render_widget(
                Paragraph::new("TODO [ESC] to return").block(Block::default().borders(Borders::ALL)),
                centered_rect(50, 50, f.size()),
            );
        }
        CurrentScreen::Search => {


        }
    }
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
