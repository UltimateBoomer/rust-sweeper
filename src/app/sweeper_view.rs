use ratatui::{
    style::{Color, Stylize},
    text::{Line, Text},
    widgets::Paragraph,
};

use crate::model::sweeper::{GameState, SweeperGame};

const NUM_TEXTS: [&str; 10] = ["　", "１", "２", "３", "４", "５", "６", "７", "８", "９"];
const NUM_COLORS: [&Color; 10] = [
    &Color::Black,
    &Color::Blue,
    &Color::Green,
    &Color::Red,
    &Color::Magenta,
    &Color::Cyan,
    &Color::Yellow,
    &Color::White,
    &Color::Gray,
    &Color::DarkGray,
];
const BOMB_TEXT: &str = "💣";
const FLAG_TEXT: &str = "🚩";
const EMPTY_TEXT: &str = "　";

pub fn draw_game(game: &SweeperGame, cursor: (isize, isize)) -> Paragraph {
    let text = Text::from_iter(game.cell_row_iter().enumerate().map(|(y, row)| {
        Line::from_iter(row.iter().enumerate().map(|(x, cell)| {
            let text = if game.state == GameState::Lose && !cell.is_revealed && cell.is_bomb {
                BOMB_TEXT.into()
            } else if cell.is_revealed {
                NUM_TEXTS[cell.mine_count as usize]
                    .bold()
                    .fg(*NUM_COLORS[cell.mine_count as usize])
            } else if cell.is_flagged {
                FLAG_TEXT.into()
            } else {
                EMPTY_TEXT.into()
            };

            if game.state == GameState::Running && (x as isize, y as isize) == cursor {
                text.on_black()
            } else if cell.is_revealed {
                text.on_dark_gray()
            } else {
                text.on_gray()
            }
        }))
    }));
    Paragraph::new(text)
}
