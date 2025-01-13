use ratatui::{
    style::{Color, Stylize},
    text::{Line, Text},
    widgets::Paragraph,
};

use crate::model::sweeper::{GameState, SweeperGame};

const NUM_TEXTS: [&str; 9] = ["　", "１", "２", "３", "４", "５", "６", "７", "８"];
const NUM_COLORS: [&Color; 9] = [
    &Color::Black,
    &Color::Blue,
    &Color::Green,
    &Color::Red,
    &Color::Magenta,
    &Color::Cyan,
    &Color::Yellow,
    &Color::White,
    &Color::Gray,
];
const BOMB_TEXT: &str = "💣";
const FLAG_TEXT: &str = "🚩";
const EMPTY_TEXT: &str = "　";

pub fn draw_game(game: &SweeperGame, cursor: (isize, isize)) -> Paragraph {
    let time_text = format!("Time: {}", game.get_elapsed_time().as_secs());
    let time_line = Line::from(time_text.bold().fg(Color::White));

    let bomb_count_line = if game.state == GameState::Win {
        Line::from("You Win!".bold().fg(Color::Green))
    } else if game.state == GameState::Lose {
        Line::from("You Lose!".bold().fg(Color::Red))
    } else {
        let bomb_count_text = format!("Remaining: {}", game.num_bombs - game.num_flags);
        Line::from(bomb_count_text.bold().fg(Color::White))
    };

    let board_text = Text::from_iter(game.cell_row_iter().enumerate().map(|(y, row)| {
        Line::from_iter(row.iter().enumerate().map(|(x, cell)| {
            let text = if game.state == GameState::Lose && cell.is_bomb {
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

            if (game.state == GameState::NotRunning || game.state == GameState::Running)
                && (x as isize, y as isize) == cursor
            {
                text.on_black()
            } else if cell.is_revealed {
                text.on_dark_gray()
            } else {
                text.on_gray()
            }
        }))
    }));

    let mut text = Text::default();
    text.lines.push(time_line);
    text.lines.push(bomb_count_line);
    text.lines.extend(board_text.lines);

    Paragraph::new(text)
}
