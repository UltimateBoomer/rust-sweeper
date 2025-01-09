use std::time::Duration;

use crate::model::sweeper::SweeperGame;

/// Controller with cursor position.
#[derive(Debug)]
pub struct SweeperController {
    pub game: Option<SweeperGame>,
    pub cursor: (isize, isize),
}

impl SweeperController {
    pub fn new() -> Self {
        Self {
            game: None,
            cursor: (0, 0),
        }
    }

    pub fn start_game(&mut self, width: usize, height: usize, bomb_count: usize) {
        self.game = Some(SweeperGame::new(width, height, bomb_count));
        self.cursor = (0, 0);
    }

    pub fn is_running(&self) -> bool {
        match self.game {
            Some(ref game) => game.state == crate::model::sweeper::GameState::Running,
            None => false,
        }
    }

    pub fn get_elapsed_time(&self) -> Duration {
        match self.game {
            Some(ref game) => game.get_elapsed_time(),
            None => Duration::ZERO,
        }
    }

    pub fn open(&mut self) {
        let (x, y) = self.cursor;
        if let Some(ref mut game) = self.game {
            game.open(x, y);
        }
    }

    pub fn flag(&mut self) {
        let (x, y) = self.cursor;
        if let Some(ref mut game) = self.game {
            game.flag(x, y);
        }
    }

    pub fn move_cursor(&mut self, dx: isize, dy: isize) {
        if let Some(ref game) = self.game {
            let (x, y) = self.cursor;
            let x = (x as isize + dx).clamp(0, game.get_width() as isize - 1);
            let y = (y as isize + dy).clamp(0, game.get_height() as isize - 1);
            self.cursor = (x, y);
        }
    }

    pub fn resign(&mut self) {
        if let Some(ref mut game) = self.game {
            game.state = crate::model::sweeper::GameState::Lose;
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_new_controller() {
        let controller = super::SweeperController::new();
        assert_eq!(controller.cursor, (0, 0));
        assert!(controller.game.is_none());
    }

    #[test]
    fn test_move_cursor() {
        let mut controller = super::SweeperController::new();
        controller.start_game(4, 4, 0);

        controller.move_cursor(1, 1);
        assert_eq!(controller.cursor, (1, 1));
        controller.move_cursor(-1, -1);
        assert_eq!(controller.cursor, (0, 0));
        controller.move_cursor(-1, -1); // should not move out of bounds
        assert_eq!(controller.cursor, (0, 0));
        controller.move_cursor(3, 3);
        assert_eq!(controller.cursor, (3, 3));
        controller.move_cursor(1, 1); // should not move out of bounds
        assert_eq!(controller.cursor, (3, 3));
    }
}
