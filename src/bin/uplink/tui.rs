use anyhow::{Context, Result};
use comms::publication::Publication;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    style::Stylize,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};

use crate::actions;

#[derive(Debug, Default)]
pub struct App {
    publications: Vec<Publication>,
    exit: bool,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        // let response = actions::get_all_publications("https://comms.yorch.dev", 1, 100)
        //     .await
        //     .context("get all publications from relay server")?;
        // let publications = response.publications();
        // self.publications = publications;
        while !self.exit {
            terminal
                .draw(|frame| self.draw(frame))
                .context("render tui app")?;
            self.handle_events().context("handle tui app events")?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let title = Line::from(" Uplink ".blue().bold());
        let block = Block::bordered().title(title);
        let text = Text::from("Hello, Uplink TUI!");

        Paragraph::new(text)
            .centered()
            .block(block)
            .render(area, buf);
    }
}
