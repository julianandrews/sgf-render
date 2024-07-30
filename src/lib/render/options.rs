use crate::errors::UsageError;

use super::{GobanRange, GobanStyle};

#[derive(Debug, Clone, Default)]
pub struct RenderOptions {
    pub node_description: NodeDescription,
    pub goban_range: GobanRange,
    pub style: GobanStyle,
    pub viewbox_width: f64,
    pub label_sides: BoardSideSet,
    pub move_number_options: Option<MoveNumberOptions>,
    pub draw_marks: bool,
    pub draw_triangles: bool,
    pub draw_circles: bool,
    pub draw_squares: bool,
    pub draw_selected: bool,
    pub draw_dimmed: bool,
    pub draw_labels: bool,
    pub draw_lines: bool,
    pub draw_arrows: bool,
    pub kifu_mode: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default, clap::Parser)]
pub struct NodeDescription {
    /// Game number to display (for multi-game files).
    #[arg(short, long, default_value_t = 0)]
    pub game_number: u64,
    /// Variation number to display (use `query` command for numbers).
    #[arg(short, long, default_value_t = 0)]
    pub variation: u64,
    /// Node number in the variation to display.
    #[arg(short, long, default_value = "last")]
    pub node_number: NodeNumber,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum NodeNumber {
    Number(u64),
    #[default]
    Last,
}

impl std::str::FromStr for NodeNumber {
    type Err = UsageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "last" => Ok(NodeNumber::Last),
            _ => {
                let n = s
                    .parse()
                    .map_err(|_| UsageError::InvalidNodeNumber(s.to_string()))?;
                Ok(NodeNumber::Number(n))
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MoveNumberOptions {
    pub start: u64,
    pub end: Option<u64>,
    pub count_from: u64,
}

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
