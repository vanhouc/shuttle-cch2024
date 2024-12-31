use std::fmt::Display;

use rand::{rngs::StdRng, Rng, SeedableRng};
use serde::Deserialize;

#[derive(Debug)]
pub enum GameError {
    ColumnFull,
    GameOver,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum GamePiece {
    Cookie,
    Milk,
}

impl Display for GamePiece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GamePiece::Cookie => write!(f, "🍪"),
            GamePiece::Milk => write!(f, "🥛"),
        }
    }
}

#[derive(Clone, Copy, Default)]
pub enum GameState {
    #[default]
    Running,
    Winner(GamePiece),
    Draw,
}

pub struct GameBoard {
    rng: StdRng,
    board: [[Option<GamePiece>; 4]; 4],
    pub state: GameState,
}

impl Default for GameBoard {
    fn default() -> Self {
        Self {
            rng: StdRng::seed_from_u64(2024),
            board: Default::default(),
            state: Default::default(),
        }
    }
}

impl Display for GameBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in self.board.iter() {
            // Print left wall
            write!(f, "⬜")?;
            for cell in row.iter() {
                if let Some(piece) = cell {
                    write!(f, "{piece}")?;
                } else {
                    write!(f, "⬛")?;
                }
            }
            // Print right wall, note that this is writeln! not write!
            writeln!(f, "⬜")?;
        }
        // Print bottom wall
        writeln!(f, "⬜⬜⬜⬜⬜⬜")?;
        // Print game state if necessary
        if let GameState::Winner(winner) = self.state {
            writeln!(f, "{winner} wins!")?;
        } else if let GameState::Draw = self.state {
            writeln!(f, "No winner.")?;
        }
        Ok(())
    }
}

impl GameBoard {
    pub fn place(&mut self, team: GamePiece, column: usize) -> Result<GameState, GameError> {
        if let GameState::Running = self.state {
            let available_index = (0..4)
                .rev()
                .find(|&row| self.board[row][column].is_none())
                .ok_or(GameError::ColumnFull)?;

            self.board[available_index][column] = Some(team);
            Ok(self.update_state())
        } else {
            Err(GameError::GameOver)
        }
    }

    pub fn randomize(&mut self) {
        for row in self.board.iter_mut() {
            for cell in row.iter_mut() {
                if self.rng.gen::<bool>() {
                    *cell = Some(GamePiece::Cookie)
                } else {
                    *cell = Some(GamePiece::Milk)
                }
            }
        }
    }

    fn update_state(&mut self) -> GameState {
        self.state = if let GameState::Running = self.state {
            if let Some(winner) = self.get_combinations().find_map(all_same) {
                GameState::Winner(winner)
            } else if self
                .get_combinations()
                .all(|combination| combination.iter().all(Option::is_some))
            {
                GameState::Draw
            } else {
                GameState::Running
            }
        } else {
            self.state
        };
        self.state
    }

    fn get_combinations(&self) -> impl Iterator<Item = Vec<Option<GamePiece>>> + '_ {
        (0..4)
            .map(|row| self.get_row(row))
            .chain((0..4).map(|column| self.get_column(column)))
            .chain(self.get_diagonals())
    }

    fn get_row(&self, row: usize) -> Vec<Option<GamePiece>> {
        self.board.iter().map(move |y| &y[row]).cloned().collect()
    }

    fn get_column(&self, column: usize) -> Vec<Option<GamePiece>> {
        self.board[column].to_vec()
    }

    fn get_diagonals(&self) -> [Vec<Option<GamePiece>>; 2] {
        [
            (0..4).map(move |i| &self.board[i][i]).cloned().collect(),
            (0..4)
                .map(move |i| &self.board[i][3 - i])
                .cloned()
                .collect(),
        ]
    }
}

fn all_same(iter: impl IntoIterator<Item = Option<GamePiece>>) -> Option<GamePiece> {
    let mut iter = iter.into_iter();
    let first = iter.next()?;
    if iter.all(|cell| cell == first) {
        first
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_game_piece() {
        assert_eq!(GamePiece::Cookie.to_string(), "🍪");
        assert_eq!(GamePiece::Milk.to_string(), "🥛");
    }

    #[test]
    fn test_display_game_board() {
        let board = GameBoard::default();
        assert_eq!(
            board.to_string(),
            "⬜⬛⬛⬛⬛⬜\n\
             ⬜⬛⬛⬛⬛⬜\n\
             ⬜⬛⬛⬛⬛⬜\n\
             ⬜⬛⬛⬛⬛⬜\n\
             ⬜⬜⬜⬜⬜⬜\n"
        );
    }

    #[test]
    fn test_place() {
        let mut game = GameBoard::default();
        game.place(GamePiece::Cookie, 0).unwrap();
        assert_eq!(
            game.to_string(),
            "⬜⬛⬛⬛⬛⬜\n\
             ⬜⬛⬛⬛⬛⬜\n\
             ⬜⬛⬛⬛⬛⬜\n\
             ⬜🍪⬛⬛⬛⬜\n\
             ⬜⬜⬜⬜⬜⬜\n"
        )
    }

    #[test]
    fn test_display_game_board_winner() {
        let mut board = GameBoard::default();
        board.board[0][0] = Some(GamePiece::Cookie);
        board.board[1][1] = Some(GamePiece::Cookie);
        board.board[2][2] = Some(GamePiece::Cookie);
        board.board[3][3] = Some(GamePiece::Cookie);
        board.update_state();
        assert_eq!(
            board.to_string(),
            "⬜🍪⬛⬛⬛⬜\n\
             ⬜⬛🍪⬛⬛⬜\n\
             ⬜⬛⬛🍪⬛⬜\n\
             ⬜⬛⬛⬛🍪⬜\n\
             ⬜⬜⬜⬜⬜⬜\n\
             🍪 wins!\n"
        );
    }

    #[test]
    fn test_game_board_draw() {
        let mut game = GameBoard::default();
        for row in 0..4 {
            if row % 2 == 0 {
                game.board[row][0] = Some(GamePiece::Milk);
                game.board[row][1] = Some(GamePiece::Cookie);
                game.board[row][2] = Some(GamePiece::Cookie);
                game.board[row][3] = Some(GamePiece::Milk);
            } else {
                game.board[row][0] = Some(GamePiece::Cookie);
                game.board[row][1] = Some(GamePiece::Milk);
                game.board[row][2] = Some(GamePiece::Milk);
                game.board[row][3] = Some(GamePiece::Cookie);
            }
        }
        game.update_state();
        assert_eq!(
            game.to_string(),
            "⬜🥛🍪🍪🥛⬜\n\
             ⬜🍪🥛🥛🍪⬜\n\
             ⬜🥛🍪🍪🥛⬜\n\
             ⬜🍪🥛🥛🍪⬜\n\
             ⬜⬜⬜⬜⬜⬜\n\
             No winner.\n"
        );
    }

    #[test]
    fn test_get_diagonals() {
        let mut board = GameBoard::default();
        for i in 0..4 {
            board.board[i][i] = Some(GamePiece::Cookie);
            board.board[i][3 - i] = Some(GamePiece::Milk);
        }
        let diagonals = board.get_diagonals();
        assert_eq!(
            diagonals[0],
            vec![
                Some(GamePiece::Cookie),
                Some(GamePiece::Cookie),
                Some(GamePiece::Cookie),
                Some(GamePiece::Cookie)
            ]
        );
        assert_eq!(
            diagonals[1],
            vec![
                Some(GamePiece::Milk),
                Some(GamePiece::Milk),
                Some(GamePiece::Milk),
                Some(GamePiece::Milk)
            ]
        );
    }
}
