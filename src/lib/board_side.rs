use crate::errors::UsageError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BoardSide {
    North,
    East,
    South,
    West,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BoardSideSet(u8);

impl BoardSideSet {
    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    pub fn contains(&self, side: BoardSide) -> bool {
        self.0 & (1 << side as u8) != 0
    }

    pub fn insert(&mut self, side: BoardSide) {
        self.0 |= 1 << side as u8
    }
}

impl std::str::FromStr for BoardSideSet {
    type Err = UsageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut value = BoardSideSet::default();
        for c in s.chars() {
            match c {
                'n' => value.insert(BoardSide::North),
                'e' => value.insert(BoardSide::East),
                's' => value.insert(BoardSide::South),
                'w' => value.insert(BoardSide::West),
                _ => return Err(UsageError::InvalidBoardSides),
            }
        }
        Ok(value)
    }
}
