use rand::seq::IteratorRandom;
use std::time::Instant;

#[derive(Debug, Clone, Copy, Default)]
pub struct Cell {
    is_bomb: bool,
    is_flagged: bool,
    is_revealed: bool,
    mine_count: u8,
}

#[derive(Debug, Clone, Default)]
pub struct Board {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameState {
    Running,
    Win,
    Lose,
}

#[derive(Debug)]
pub struct SweeperGame {
    pub board: Board,
    pub num_bombs: usize,
    pub num_revealed: usize,
    pub num_flags: usize,
    pub state: GameState,
    pub start_time: Instant,
    pub cursor: (isize, isize),
}

impl SweeperGame {
    /// Initialize and start a new game.
    pub fn new(width: usize, height: usize, num_bombs: usize) -> Self {
        let mut cells = vec![Cell::default(); width * height];
        // Distribute bombs
        for i in (0..width * height).choose_multiple(&mut rand::thread_rng(), num_bombs) {
            cells[i].is_bomb = true;
        }

        let board = Board {
            width,
            height,
            cells,
        };

        Self {
            board,
            num_bombs,
            num_revealed: 0,
            num_flags: 0,
            state: GameState::Running,
            start_time: Instant::now(),
            cursor: (0, 0),
        }
    }

    /// Move the cursor by the given offset.
    pub fn move_cursor(&mut self, x: isize, y: isize) {
        let cx = (self.cursor.0 + x).clamp(0, (self.board.width - 1) as isize);
        let cy = (self.cursor.1 + y).clamp(0, (self.board.height - 1) as isize);
        self.cursor = (cx, cy);
    }

    /// Unveil the cell at the given coordinate.
    pub fn open(&mut self, x: isize, y: isize) -> GameState {
        let cell_index = self.cell_index(x, y);
        if let Some(cell_index) = cell_index {
            let cell = &self.board.cells[cell_index];
            if !cell.is_revealed {
                if cell.is_bomb {
                    self.state = GameState::Lose;
                } else {
                    self.reveal_cell(cell_index);
                }
            }
        }
        self.state
    }

    pub fn flag(&mut self, x: isize, y: isize) {
        let cell_index = self.cell_index(x, y);
        if let Some(cell_index) = cell_index {
            let cell = &mut self.board.cells[cell_index];
            if !cell.is_revealed {
                cell.is_flagged = !cell.is_flagged;
                if cell.is_flagged {
                    self.num_flags += 1;
                } else {
                    self.num_flags -= 1;
                }
            }
        }
    }

    fn reveal_cell(&mut self, cell_index: usize) {
        let mut cell_stack = vec![cell_index];

        while let Some(cell_index) = cell_stack.pop() {
            {
                let cell = &mut self.board.cells[cell_index];
                cell.is_revealed = true;
            }

            self.num_revealed += 1;

            if self.num_revealed >= self.board.width * self.board.height - self.num_bombs {
                self.state = GameState::Win;
                break;
            }

            let adjacent = self.adjacent_unopened_cells(cell_index);
            let mine_count = adjacent
                .iter()
                .filter(|&&i| self.board.cells[i].is_bomb)
                .count();
            {
                let cell = &mut self.board.cells[cell_index];
                cell.mine_count = mine_count as u8;
            }

            if mine_count == 0 {
                for &j in &adjacent {
                    cell_stack.push(j);
                }
            }
        }
    }

    fn cell_index(&self, x: isize, y: isize) -> Option<usize> {
        if x >= self.board.width as isize || y >= self.board.height as isize || x < 0 || y < 0 {
            None
        } else {
            Some(y as usize * self.board.width + x as usize)
        }
    }

    fn adjacent_unopened_cells(&self, cell_index: usize) -> Vec<usize> {
        let x = cell_index % self.board.width;
        let y = cell_index / self.board.width;
        (-1..=1)
            .flat_map(|i| (-1..=1).map(move |j| (j, i)))
            .filter_map(|(i, j)| {
                if i == 0 && j == 0 {
                    None
                } else {
                    self.cell_index(x as isize + i, y as isize + j)
                        .filter(|&index| !self.board.cells[index].is_revealed)
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_game() {
        let game = SweeperGame::new(10, 10, 20);
        assert_eq!(game.board.width, 10);
        assert_eq!(game.board.height, 10);
        assert_eq!(game.num_bombs, 20);
        assert_eq!(game.num_revealed, 0);
        assert_eq!(game.num_flags, 0);
        assert_eq!(game.state, GameState::Running);
        assert_eq!(game.cursor, (0, 0));

        // Test sum of bombs
        let num_bombs = game.board.cells.iter().filter(|&cell| cell.is_bomb).count();
        assert_eq!(num_bombs, game.num_bombs);
    }

    #[test]
    fn test_cell_index() {
        let game = SweeperGame::new(10, 10, 0);
        assert_eq!(game.cell_index(0, 0), Some(0));
        assert_eq!(game.cell_index(9, 9), Some(99));
        assert_eq!(game.cell_index(10, 10), None);
        assert_eq!(game.cell_index(-1, -1), None);
    }

    #[test]
    fn test_adjacent_unopened_cells() {
        let game = SweeperGame::new(10, 10, 0);
        assert_eq!(game.adjacent_unopened_cells(0), vec![1, 10, 11]);
        assert_eq!(game.adjacent_unopened_cells(9), vec![8, 18, 19]);
        assert_eq!(game.adjacent_unopened_cells(90), vec![80, 81, 91]);
        assert_eq!(game.adjacent_unopened_cells(99), vec![88, 89, 98]);
        assert_eq!(game.adjacent_unopened_cells(1), vec![0, 2, 10, 11, 12]);
        assert_eq!(game.adjacent_unopened_cells(10), vec![0, 1, 11, 20, 21]);
        assert_eq!(
            game.adjacent_unopened_cells(11),
            vec![0, 1, 2, 10, 12, 20, 21, 22]
        );
    }

    #[test]
    fn test_open_simple() {
        let mut game = SweeperGame::new(10, 10, 0);

        // Bombs
        for i in [10, 11] {
            game.board.cells[i].is_bomb = true;
        }

        assert_eq!(game.open(0, 0), GameState::Running);
        assert_eq!(game.num_revealed, 1);
        assert_eq!(game.board.cells[0].is_revealed, true);
        assert_eq!(game.board.cells[0].mine_count, 2);
    }

    #[test]
    fn test_open_bomb() {
        let mut game = SweeperGame::new(10, 10, 0);
        game.board.cells[0].is_bomb = true;

        assert_eq!(game.open(0, 0), GameState::Lose);
    }

    #[test]
    fn test_open_multiple() {
        let mut game = SweeperGame::new(10, 10, 0);

        // Bombs
        // 0 1 .
        // 2 3 x
        // x x .
        for i in [20, 21, 12] {
            game.board.cells[i].is_bomb = true;
        }

        assert_eq!(game.open(0, 0), GameState::Running);
        assert_eq!(game.num_revealed, 4);
        assert_eq!(game.board.cells[0].is_revealed, true);
        assert_eq!(game.board.cells[0].mine_count, 0);
        assert_eq!(game.board.cells[1].is_revealed, true);
        assert_eq!(game.board.cells[1].mine_count, 1);
        assert_eq!(game.board.cells[10].is_revealed, true);
        assert_eq!(game.board.cells[10].mine_count, 2);
        assert_eq!(game.board.cells[11].is_revealed, true);
        assert_eq!(game.board.cells[11].mine_count, 3);
    }
}
