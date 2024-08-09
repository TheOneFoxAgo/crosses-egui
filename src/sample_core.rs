use std::error::Error;
use std::fmt::Display;

use crosses_core::board_manager::{BoardError, BoardManager, Cell, CellKind};
use crosses_core::player_manager::{self, GameOver, GameState, LoseData};
use serde::{Deserialize, Serialize};

pub mod sample_board_manager;
pub mod sample_cell;

#[derive(Serialize, Deserialize)]
pub struct CrossesCore {
    pub board_manager: sample_board_manager::SampleBoardManager,
    pub player_manager: player_manager::PlayerManager<[Option<LoseData>; 2]>,
    pub log: Vec<(usize, usize)>,
}
impl CrossesCore {
    pub fn make_move(&mut self, x: usize, y: usize) -> Result<(), CrossesError> {
        if let GameState::Ended(game_over) = self.player_manager.game_state() {
            return Err(CrossesError::PlayerError(game_over));
        }
        self.board_manager
            .make_move((x, y), self.player_manager.current_player() == 1)?;
        self.player_manager.advance(
            |p| self.board_manager.moves_counter[p] == 0,
            |p| self.board_manager.crosses_counter[p] == 0,
        );
        self.log.truncate(self.player_manager.current_move() - 1);
        self.log.push((x, y));
        Ok(())
    }
    pub fn can_back(&self) -> bool {
        self.player_manager.current_move() != 0
    }
    pub fn back(&mut self) -> Result<(), CrossesError> {
        let index = *self
            .log
            .get(self.player_manager.current_move() - 1)
            .ok_or(CrossesError::BackError)?;
        let cell = self.board_manager.get(index);
        if let CellKind::Empty | CellKind::Border = cell.kind() {
            return Err(CrossesError::CorruptedLog);
        }
        let player = cell.player();
        self.board_manager.cancel_move(index, || !player)?;
        self.player_manager.reverse(player as usize);
        Ok(())
    }
    pub fn can_forward(&self) -> bool {
        self.player_manager.current_move() != self.log.len()
    }
    pub fn forward(&mut self) -> Result<(), CrossesError> {
        let index = *self
            .log
            .get(self.player_manager.current_move() - 1)
            .ok_or(CrossesError::ForwardError)?;
        self.board_manager
            .make_move(index, self.player_manager.current_player() == 1)?;
        self.player_manager.advance(
            |p| self.board_manager.moves_counter[p] == 0,
            |p| self.board_manager.crosses_counter[p] == 0,
        );
        Ok(())
    }
}

#[derive(Debug)]
pub enum CrossesError {
    BoardError(BoardError),
    PlayerError(GameOver),
    BackError,
    ForwardError,
    CorruptedLog,
}
impl Display for CrossesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CrossesError::BoardError(be) => write!(f, "{}", be),
            CrossesError::PlayerError(pe) => write!(f, "{}", pe),
            CrossesError::BackError => write!(f, "there's no going back"),
            CrossesError::ForwardError => write!(f, "nothing ahead"),
            CrossesError::CorruptedLog => write!(f, "log was corrupterd"),
        }
    }
}
impl Error for CrossesError {}
impl From<BoardError> for CrossesError {
    fn from(value: BoardError) -> Self {
        Self::BoardError(value)
    }
}
impl From<GameOver> for CrossesError {
    fn from(value: GameOver) -> Self {
        Self::PlayerError(value)
    }
}
