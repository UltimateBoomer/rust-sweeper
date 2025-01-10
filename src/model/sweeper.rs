use rand::seq::IteratorRandom;
use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

#[derive(Debug, Clone, Copy, Default)]
pub struct Cell {
    pub is_bomb: bool,
    pub is_flagged: bool,
    pub is_revealed: bool,
    pub mine_count: u8,
}

#[derive(Debug, Clone, Default)]
pub struct Board {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Cell>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameState {
    NotRunning,
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
    pub start_time: Option<Instant>,
    pub total_time: Duration,
}

impl SweeperGame {
    /// Initialize and start a new game.
    pub fn new(width: usize, height: usize, num_bombs: usize) -> Self {
        let cells = vec![Cell::default(); width * height];

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
            state: GameState::NotRunning,
            start_time: None,
            total_time: Duration::ZERO,
        }
    }

    /// Generate board with bombs, excluding the given cell.
    pub fn generate_board(&mut self, x: isize, y: isize) {
        let mut rng = rand::thread_rng();
        let bomb_indices = (0..self.board.cells.len())
            .filter(|&i| i != self.cell_index(x, y).unwrap())
            .choose_multiple(&mut rng, self.num_bombs);

        for i in bomb_indices {
            self.board.cells[i].is_bomb = true;
        }
    }

    /// Unveil the cell at the given coordinate.
    pub fn open(&mut self, x: isize, y: isize) -> GameState {
        if let Some(cell_index) = self.cell_index(x, y) {
            let cell = &self.board.cells[cell_index];
            if cell.is_revealed {
                // Reveal surrounding cells if the number of flags around the cell is equal to the bomb count
                self.reveal_adjacent_cells(cell_index);
            } else {
                self.reveal_cell(cell_index);
            }
        }

        if self.num_revealed >= self.board.width * self.board.height - self.num_bombs {
            self.state = GameState::Win;
        }

        if self.state != GameState::Running {
            self.end_game();
        }

        self.state
    }

    /// Toggle flag on the cell at the given coordinate.
    pub fn flag(&mut self, x: isize, y: isize) {
        if let Some(cell_index) = self.cell_index(x, y) {
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

    pub fn get_width(&self) -> usize {
        self.board.width
    }

    pub fn get_height(&self) -> usize {
        self.board.height
    }

    pub fn is_valid_coordinate(&self, x: isize, y: isize) -> bool {
        x >= 0 && x < self.board.width as isize && y >= 0 && y < self.board.height as isize
    }

    pub fn get_cell(&self, x: isize, y: isize) -> Option<&Cell> {
        self.cell_index(x, y).map(|index| &self.board.cells[index])
    }

    /// Get an iterator over the board row slices along with their coordinates.
    pub fn cell_row_iter(&self) -> impl Iterator<Item = &[Cell]> {
        self.board.cells.chunks(self.board.width)
    }

    pub fn get_elapsed_time(&self) -> Duration {
        match self.state {
            GameState::Running => match self.start_time {
                Some(start_time) => start_time.elapsed(),
                None => Duration::ZERO,
            },
            GameState::Win | GameState::Lose => self.total_time,
            _ => Duration::ZERO,
        }
    }

    pub fn start(&mut self) {
        self.state = GameState::Running;
        self.start_time = Some(Instant::now());
    }

    fn end_game(&mut self) {
        if let Some(start_time) = self.start_time {
            self.total_time = start_time.elapsed();
        }
    }

    fn reveal_cell(&mut self, cell_index: usize) {
        self.set_revealed(cell_index);
        if self.board.cells[cell_index].is_bomb {
            self.state = GameState::Lose;
            return;
        }

        self.reveal_cell_queue(VecDeque::from([cell_index]));
    }

    fn reveal_adjacent_cells(&mut self, cell_index: usize) {
        let adjacent = self.adjacent_cells(cell_index);
        let flag_count = adjacent
            .iter()
            .filter(|&&i| self.board.cells[i].is_flagged)
            .count();
        if flag_count != self.board.cells[cell_index].mine_count as usize {
            return;
        }

        let cell_q = VecDeque::from_iter(
            adjacent
                .iter()
                .filter(|&&i| !self.board.cells[i].is_flagged)
                .copied(),
        );
        for &i in &cell_q {
            self.set_revealed(i);
            if self.board.cells[i].is_bomb {
                self.state = GameState::Lose;
                return;
            }
        }
        self.reveal_cell_queue(cell_q);
    }

    fn reveal_cell_queue(&mut self, mut cell_q: VecDeque<usize>) {
        while let Some(cell_index) = cell_q.pop_front() {
            let adjacent = self.adjacent_cells(cell_index);
            let mine_count = adjacent
                .iter()
                .filter(|&&i| self.board.cells[i].is_bomb)
                .count();
            self.board.cells[cell_index].mine_count = mine_count as u8;

            if mine_count == 0 {
                for j in self.adjacent_cells(cell_index) {
                    if self.board.cells[j].is_revealed || self.board.cells[j].is_flagged {
                        continue;
                    }

                    self.set_revealed(j);
                    cell_q.push_back(j);
                }
            }
        }
    }

    fn cell_index(&self, x: isize, y: isize) -> Option<usize> {
        if self.is_valid_coordinate(x, y) {
            Some(y as usize * self.board.width + x as usize)
        } else {
            None
        }
    }

    fn set_revealed(&mut self, cell_index: usize) {
        if self.board.cells[cell_index].is_revealed {
            return;
        }
        self.board.cells[cell_index].is_revealed = true;
        self.num_revealed += 1;
    }

    fn adjacent_cells(&self, cell_index: usize) -> Vec<usize> {
        let x = cell_index % self.board.width;
        let y = cell_index / self.board.width;
        (-1..=1)
            .flat_map(|i| (-1..=1).map(move |j| (j, i)))
            .filter(|&(i, j)| i != 0 || j != 0)
            .filter_map(|(i, j)| self.cell_index(x as isize + i, y as isize + j))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_game() {
        let mut game = SweeperGame::new(10, 10, 20);
        game.generate_board(0, 0);
        assert_eq!(game.state, GameState::NotRunning);
        game.start();
        assert_eq!(game.board.width, 10);
        assert_eq!(game.board.height, 10);
        assert_eq!(game.num_bombs, 20);
        assert_eq!(game.num_revealed, 0);
        assert_eq!(game.num_flags, 0);
        assert_eq!(game.state, GameState::Running);

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
    fn test_adjacent_cells() {
        let game = SweeperGame::new(10, 10, 0);
        assert_eq!(game.adjacent_cells(0), vec![1, 10, 11]);
        assert_eq!(game.adjacent_cells(9), vec![8, 18, 19]);
        assert_eq!(game.adjacent_cells(90), vec![80, 81, 91]);
        assert_eq!(game.adjacent_cells(99), vec![88, 89, 98]);
        assert_eq!(game.adjacent_cells(1), vec![0, 2, 10, 11, 12]);
        assert_eq!(game.adjacent_cells(10), vec![0, 1, 11, 20, 21]);
        assert_eq!(game.adjacent_cells(11), vec![0, 1, 2, 10, 12, 20, 21, 22]);
    }

    #[test]
    fn test_open_simple() {
        let mut game = SweeperGame::new(10, 10, 0);

        // Bombs
        for i in [10, 11] {
            game.board.cells[i].is_bomb = true;
            game.num_bombs += 1;
        }
        game.start();

        assert_eq!(game.open(0, 0), GameState::Running);
        assert_eq!(game.num_revealed, 1);
        assert_eq!(game.board.cells[0].is_revealed, true);
        assert_eq!(game.board.cells[0].mine_count, 2);
    }

    #[test]
    fn test_open_bomb() {
        let mut game = SweeperGame::new(10, 10, 0);
        game.board.cells[0].is_bomb = true;
        game.start();

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
            game.num_bombs += 1;
        }
        game.start();

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

    #[test]
    fn test_open_already_revealed() {
        let mut game = SweeperGame::new(10, 10, 0);

        // Layout
        // 1 F .
        // x . .
        // . . .
        for i in [10] {
            game.board.cells[i].is_bomb = true;
            game.num_bombs += 1;
        }
        game.start();

        assert_eq!(game.open(0, 0), GameState::Running);
        game.flag(1, 0);
        assert_eq!(game.open(0, 0), GameState::Lose);

        assert_eq!(game.num_revealed, 2);
        assert_eq!(game.board.cells[0].is_revealed, true);
        assert_eq!(game.board.cells[1].is_revealed, false);
        assert_eq!(game.board.cells[10].is_revealed, true);
    }

    #[test]
    fn test_open_win() {
        let mut game = SweeperGame::new(10, 10, 0);

        // Layout
        // x 1 0
        // 1 1 0
        // 0 0 0
        game.board.cells[0].is_bomb = true;
        game.num_bombs = 1;
        game.start();

        assert_eq!(game.open(2, 2), GameState::Win);
    }
}
