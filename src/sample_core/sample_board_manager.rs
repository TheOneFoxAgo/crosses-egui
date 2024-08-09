use super::sample_cell::SampleCell;
use crosses_core::board_manager::{self, init, Cell, CellKind};
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, ops::ControlFlow};

#[derive(Clone, Serialize, Deserialize)]
pub struct SampleBoardManager {
    pub board: [[SampleCell; 16]; 16],
    pub max_x: usize,
    pub max_y: usize,
    pub moves_counter: [usize; 2],
    pub crosses_counter: [usize; 2],
}

impl SampleBoardManager {
    pub fn new(max_x: usize, max_y: usize) -> Self {
        assert!(max_x > 1 && max_y > 1);
        let mut manager = Self {
            board: [[SampleCell::BORDER; 16]; 16],
            max_x,
            max_y,
            moves_counter: [0, 0],
            crosses_counter: [1, 1],
        };
        for x in 0..max_x {
            for y in 0..max_y {
                manager.board[x][y] = SampleCell {
                    data: 0b00100000,
                    activity: 0,
                };
            }
        }
        manager.board[0][0] = SampleCell {
            data: 0b01000000,
            activity: 0,
        };
        init(&mut manager, (0, 0), false);
        manager.board[max_x - 1][max_y - 1] = SampleCell {
            data: 0b01010000,
            activity: 0,
        };
        init(&mut manager, (max_x - 1, max_y - 1), true);
        manager
    }
    pub fn clear_checked(&mut self) {
        for column in self.board.iter_mut() {
            for cell in column.iter_mut() {
                cell.set_checked(false)
            }
        }
    }
}

impl board_manager::BoardManager for SampleBoardManager {
    type Index = (usize, usize);

    type Cell = SampleCell;

    fn adjacent(&mut self, index: Self::Index) -> impl IntoIterator<Item = Self::Index> {
        [
            (index.0.wrapping_sub(1), index.1.wrapping_sub(1)),
            (index.0, index.1.wrapping_sub(1)),
            (index.0.wrapping_add(1), index.1.wrapping_sub(1)),
            (index.0.wrapping_sub(1), index.1),
            (index.0.wrapping_add(1), index.1),
            (index.0.wrapping_sub(1), index.1.wrapping_add(1)),
            (index.0, index.1.wrapping_add(1)),
            (index.0.wrapping_add(1), index.1.wrapping_add(1)),
        ]
    }

    fn get(&self, index: Self::Index) -> Self::Cell {
        if index.0 < self.max_x && index.1 < self.max_y {
            self.board[index.0][index.1]
        } else {
            SampleCell::BORDER
        }
    }

    fn get_mut(&self, index: Self::Index) -> &mut Self::Cell {
        todo!()
    }

    type Adjacent;

    fn traverse(
        &mut self,
        index: Self::Index,
        mut action: impl FnMut(&mut Self, Self::Index) -> std::ops::ControlFlow<Self::Index, ()>,
    ) -> Option<Self::Index> {
        self.clear_checked();
        let mut result = None;
        let mut cell = self.get(index);
        let player = cell.player();
        let mut queue = VecDeque::new();
        queue.push_back(index);
        cell.set_checked(true);
        self.set(index, cell);
        action(self, index);
        while let Some(index) = queue.pop_front() {
            for adjacent_index in SampleBoardManager::adjacent(index) {
                if !self.get(adjacent_index).is_checked() {
                    if let ControlFlow::Break(new_index) = action(self, adjacent_index) {
                        result = Some(new_index);
                        break;
                    }
                    let mut cell = self.get(adjacent_index);
                    if cell.kind() == CellKind::Filled && cell.player() == player {
                        queue.push_back(adjacent_index)
                    }
                    cell.set_checked(true);
                    self.set(adjacent_index, cell);
                }
            }
        }
        return result;
    }

    fn update_counter(
        &mut self,
        player: board_manager::Player<Self>,
        kind: board_manager::CounterKind,
        op: board_manager::CounterOp,
    ) {
        todo!()
    }

    fn revive(&mut self, index: Self::Index, mut revive: impl FnMut(&mut Self, Self::Index)) {
        self.traverse(index, |manager, action| {
            revive(manager, action);
            ControlFlow::Continue(())
        });
    }

    fn kill(&mut self, index: Self::Index, mut kill: impl FnMut(&mut Self, Self::Index)) {
        self.traverse(index, |manager, action| {
            kill(manager, action);
            ControlFlow::Continue(())
        });
    }

    fn search(
        &mut self,
        index: Self::Index,
        search: impl FnMut(&mut Self, Self::Index) -> ControlFlow<Self::Index, ()>,
    ) -> Option<Self::Index> {
        self.traverse(index, search)
    }

    fn make_move(
        &mut self,
        index: Self::Index,
        player: board_manager::Player<Self>,
    ) -> Result<(), board_manager::BoardError> {
        let cell = self.get_mut(index);
        match cell.kind() {
            CellKind::Empty => {
                if !cell.is_active(player) {
                    return Err(board_manager::BoardError::OutOfReach);
                }
                cell.cross_out(player);
                let should_set_important = activate_around(self, index, player);
                self.get_mut(index).set_important(should_set_important);
                self.update_counter(
                    player,
                    board_manager::CounterKind::Crosses,
                    board_manager::CounterOp::Add,
                );
                self.update_counter(
                    player,
                    board_manager::CounterKind::Moves,
                    board_manager::CounterOp::Sub,
                );
            }
            CellKind::Cross => {
                if cell.player() == player {
                    return Err(board_manager::BoardError::SelfFill);
                }
                if !cell.is_active(player) {
                    return Err(board_manager::BoardError::OutOfReach);
                }
                let was_important = cell.is_important();
                let previous_player = cell.player();
                cell.fill(player);
                self.update_counter(
                    player,
                    board_manager::CounterKind::Crosses,
                    board_manager::CounterOp::Sub,
                );
                self.update_counter(
                    previous_player,
                    board_manager::CounterKind::Moves,
                    board_manager::CounterOp::Sub,
                );
                deactivate_around(self, index, previous_player, was_important);
                let mut important = false;
                if !is_alive_filled_around(self, index, player) {
                    important = true;
                    mark_adjacent_as_important(self, index, player, CellKind::Cross);
                }
                let should_set_important = activate_around(self, index, player) || important;
                self.get_mut(index).set_important(should_set_important);
            }
            CellKind::Filled => return Err(board_manager::BoardError::DoubleFill),
            CellKind::Border => return Err(board_manager::BoardError::BorderHit),
        };
        Ok(())
    }

    fn cancel_move(
        &mut self,
        index: Self::Index,
        mut get_player: impl FnMut() -> board_manager::Player<Self>,
    ) -> Result<(), board_manager::BoardError> {
        let cell = self.get_mut(index);
        match cell.kind() {
            CellKind::Empty => return Err(board_manager::BoardError::EmptyCancel),
            CellKind::Cross => {
                let was_important = cell.is_important();
                let previous_player = cell.player();
                cell.remove_cross();
                self.update_counter(
                    previous_player,
                    board_manager::CounterKind::Crosses,
                    board_manager::CounterOp::Sub,
                );
                self.update_counter(
                    previous_player,
                    board_manager::CounterKind::Moves,
                    board_manager::CounterOp::Add,
                );
                deactivate_around(self, index, previous_player, was_important);
            }
            CellKind::Filled => {
                let player = get_player();
                let was_important = cell.is_important();
                let previous_player = cell.player();
                cell.remove_fill(player);
                self.update_counter(
                    previous_player,
                    board_manager::CounterKind::Moves,
                    board_manager::CounterOp::Add,
                );
                self.update_counter(
                    player,
                    board_manager::CounterKind::Crosses,
                    board_manager::CounterOp::Sub,
                );
                deactivate_around(self, index, previous_player, was_important);
                let should_set_important = activate_around(self, index, player);
                self.get_mut(index).set_important(should_set_important);
            }
            CellKind::Border => return Err(board_manager::BoardError::BorderHit),
        }
        Ok(())
    }
}

impl Default for SampleBoardManager {
    fn default() -> Self {
        Self::new(10, 10)
    }
}
