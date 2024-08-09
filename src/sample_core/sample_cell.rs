use crosses_core::board_manager::{self, ActivationStatus, CellKind};
use serde::{Deserialize, Serialize};

const BORDER: u8 = 0b00;
const EMPTY: u8 = 0b01;
const CROSS: u8 = 0b10;
const FILLED: u8 = 0b11;

const TYPE: u8 = 5;
const PLAYER: u8 = 4;
const IMPORTANCE: u8 = 3;
const ALIVE: u8 = 2;
const OVERHEAT: u8 = 1;
const CHECKED: u8 = 0;

const ACTIVITY_SIZE: u8 = 2;
/// Эта клетка имеет такую структуру:
/// Резерв Тип Игрок Важность Живость Перегретость Проверенность
/// (0)    00  0     0        0       0            0
/// Плюс ещё сверху активность:
/// Резерв Синий Красный
/// (0000) 00    00
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct SampleCell {
    pub data: u8,
    pub activity: u8,
}
impl board_manager::Cell for SampleCell {
    type Player = bool;

    fn kind(self) -> board_manager::CellKind {
        match self.data >> TYPE & 0b11 {
            BORDER => CellKind::Border,
            EMPTY => CellKind::Empty,
            CROSS => CellKind::Cross,
            FILLED => CellKind::Filled,
            _ => unreachable!(),
        }
    }

    fn player(self) -> Self::Player {
        self.data >> PLAYER & 1 == 1
    }

    fn is_active(self, player: Self::Player) -> bool {
        assert!(self.kind() == CellKind::Empty || self.kind() == CellKind::Cross);
        if self.kind() == CellKind::Cross && self.player() == player {
            return false;
        }
        ActivityParser::new(self.activity, player).activity != 0
    }

    fn is_important(self) -> bool {
        assert!(self.kind() == CellKind::Cross || self.kind() == CellKind::Filled);
        self.get(IMPORTANCE)
    }

    fn set_important(&mut self, new: bool) {
        assert!(self.kind() == CellKind::Cross || self.kind() == CellKind::Filled);
        self.set(IMPORTANCE, new);
    }

    fn is_alive(self) -> bool {
        assert_eq!(self.kind(), CellKind::Filled);
        self.get(ALIVE)
    }

    fn set_alive(&mut self, new: bool) {
        assert_eq!(self.kind(), CellKind::Filled);
        self.set(ALIVE, new);
    }

    fn cross_out(&mut self, player: Self::Player) {
        assert_eq!(self.kind(), CellKind::Empty);
        self.set_type(CROSS);
        self.set_player(player);
    }

    fn fill(&mut self, player: Self::Player) {
        assert_eq!(self.kind(), CellKind::Cross);
        self.set_type(FILLED);
        self.set_player(player);
        self.set_alive(true);
    }

    fn remove_fill(&mut self, player: Self::Player) {
        assert_eq!(self.kind(), CellKind::Filled);
        self.set_type(CROSS);
        self.set_player(player);
    }

    fn remove_cross(&mut self) {
        assert_eq!(self.kind(), CellKind::Cross);
        self.set_type(EMPTY);
    }

    fn activate(&mut self, player: Self::Player) -> ActivationStatus {
        let data = ActivityParser::new(self.activity, player);
        if data.activity >= data.filler - 1 {
            self.activity |= data.filler << data.offset;
            ActivationStatus::Overheat
        } else {
            self.activity += 1 << data.offset;
            ActivationStatus::Regular
        }
    }

    fn deactivate(&mut self, player: Self::Player) -> ActivationStatus {
        let data = ActivityParser::new(self.activity, player);
        if data.activity <= 1 {
            self.activity &= !data.filler << data.offset;
            ActivationStatus::Zero
        } else {
            self.activity -= 1 << data.offset;
            ActivationStatus::Regular
        }
    }

    fn reset_activity(&mut self) {
        self.activity = 0;
    }

    fn is_overheated(self) -> bool {
        self.get(OVERHEAT)
    }

    fn set_overheat(&mut self, new: bool) {
        self.set(OVERHEAT, new);
    }
}

impl SampleCell {
    pub const BORDER: Self = Self {
        data: 0,
        activity: 0,
    };
    pub fn is_checked(self) -> bool {
        self.get(CHECKED)
    }
    pub fn set_checked(&mut self, new: bool) {
        self.set(CHECKED, new)
    }
    pub fn activity(&self, player: bool) -> u8 {
        ActivityParser::new(self.activity, player).activity
    }
    fn set_type(&mut self, new: u8) {
        self.data &= !(0b11 << TYPE);
        self.data |= new << TYPE;
    }
    fn set_player(&mut self, new: bool) {
        self.set(PLAYER, new)
    }
    fn set(&mut self, offset: u8, new: bool) {
        if new {
            self.data |= 1 << offset;
        } else {
            self.data &= !(1 << offset);
        }
    }
    fn get(self, offset: u8) -> bool {
        self.data >> offset & 1 == 1
    }
}

struct ActivityParser {
    offset: u8,
    filler: u8,
    activity: u8,
}
impl ActivityParser {
    fn new(raw_activity: u8, player: bool) -> Self {
        let offset = ACTIVITY_SIZE * player as u8;
        let filler = (1 << ACTIVITY_SIZE) - 1;
        let activity = (raw_activity >> offset) & filler;
        Self {
            offset,
            filler,
            activity,
        }
    }
}
