use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::text::Text;
use ratatui::widgets::Paragraph;
use ratatui::{style::Stylize, text::Line, widgets::Block, DefaultTerminal, Frame};
use std::fmt;
use std::time::Duration;
use sweeper_controller::SweeperController;
use sweeper_view::draw_game;

pub mod sweeper_controller;
pub mod sweeper_view;

#[derive(Debug, PartialEq)]
enum Difficulty {
    Beginner,
    Intermediate,
    Expert,
}

impl Difficulty {
    fn setting(&self) -> GameSetting {
        match self {
            Difficulty::Beginner => GameSetting {
                width: 10,
                height: 10,
                bomb_count: 10,
            },
            Difficulty::Intermediate => GameSetting {
                width: 16,
                height: 16,
                bomb_count: 40,
            },
            Difficulty::Expert => GameSetting {
                width: 30,
                height: 16,
                bomb_count: 99,
            },
        }
    }

    fn next(&self) -> Self {
        match self {
            Difficulty::Beginner => Difficulty::Intermediate,
            Difficulty::Intermediate => Difficulty::Expert,
            Difficulty::Expert => Difficulty::Beginner,
        }
    }
}

impl fmt::Display for Difficulty {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Difficulty::Beginner => write!(f, "Beginner"),
            Difficulty::Intermediate => write!(f, "Intermediate"),
            Difficulty::Expert => write!(f, "Expert"),
        }
    }
}

#[derive(Debug, PartialEq)]
enum AppState {
    Menu,
    InGame,
    Exit,
}

#[derive(Debug)]
struct GameSetting {
    width: usize,
    height: usize,
    bomb_count: usize,
}

#[derive(Debug)]
pub struct App {
    controller: SweeperController,
    state: AppState,
    difficulty: Difficulty,
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        Self {
            controller: SweeperController::new(),
            state: AppState::Menu,
            difficulty: Difficulty::Beginner,
        }
    }

    /// Main application loop
    pub fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while self.state != AppState::Exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_crossterm_events()?;
        }
        Ok(())
    }

    fn start_game(&mut self) {
        self.state = AppState::InGame;
        self.controller.start_game(
            self.difficulty.setting().width,
            self.difficulty.setting().height,
            self.difficulty.setting().bomb_count,
        );
    }

    /// Renders the user interface.
    ///
    /// This is where you add new widgets. See the following resources for more information:
    /// - <https://docs.rs/ratatui/latest/ratatui/widgets/index.html>
    /// - <https://github.com/ratatui/ratatui/tree/master/examples>
    fn draw(&mut self, frame: &mut Frame) {
        let title = Line::from("Rust Sweeper ".blue().bold()).centered();

        frame.render_widget(
            match self.state {
                AppState::Menu => {
                    let difficulty_text = format!("Difficulty: {} ('d')", self.difficulty);
                    let difficulty_line = Line::from(difficulty_text.bold());
                    let start_line = Line::from("Press 'n' to start a new game".bold());
                    let quit_line = Line::from("Press 'q' to quit".bold());
                    let lines = vec![difficulty_line, start_line, quit_line];
                    Paragraph::new(Text::from(lines))
                }
                AppState::InGame => draw_game(
                    self.controller.game.as_ref().unwrap(),
                    self.controller.cursor,
                ),
                _ => Paragraph::new(Text::from(Line::from("Goodbye!"))),
            }
            .block(Block::bordered().title(title))
            .centered(),
            frame.area(),
        );
    }

    /// Reads the crossterm events and updates the state of [`App`].
    ///
    /// If your application needs to perform work in between handling events, you can use the
    /// [`event::poll`] function to check if there are any events available with a timeout.
    fn handle_crossterm_events(&mut self) -> Result<()> {
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                // it's important to check KeyEventKind::Press to avoid handling key release events
                Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
                Event::Mouse(_) => {}
                Event::Resize(_, _) => {}
                _ => {}
            }
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q')) => {
                if self.controller.is_running() {
                    self.controller.resign();
                } else {
                    self.quit();
                }
            }
            (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            (_, KeyCode::Char('n')) => self.start_game(),
            _ => match self.state {
                AppState::Menu => self.on_menu_key_event(key),
                AppState::InGame => self.on_game_key_event(key),
                _ => {}
            },
        }
    }

    fn on_menu_key_event(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('d') => self.difficulty = self.difficulty.next(),
            _ => {}
        }
    }

    fn on_game_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Left) => self.controller.move_cursor(-1, 0),
            (_, KeyCode::Right) => self.controller.move_cursor(1, 0),
            (_, KeyCode::Up) => self.controller.move_cursor(0, -1),
            (_, KeyCode::Down) => self.controller.move_cursor(0, 1),
            (_, KeyCode::Char(' ')) => {
                if self.controller.is_running() {
                    self.controller.open();
                }
            }
            (_, KeyCode::Char('f')) => {
                if self.controller.is_running() {
                    self.controller.flag();
                }
            }
            _ => {}
        }
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.state = AppState::Exit;
    }
}
