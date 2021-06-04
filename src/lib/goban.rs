use sgf_parse::{go, SgfNode};
use std::collections::{HashMap, HashSet, VecDeque};

pub struct Goban {
    pub size: (u8, u8),
    pub stones: HashMap<(u8, u8), StoneColor>,
    pub move_numbers: HashMap<(u8, u8), Vec<u64>>,
    pub move_number: u64,
    pub black_captures: u64,
    pub white_captures: u64,
    pub marks: HashSet<(u8, u8)>,
    pub triangles: HashSet<(u8, u8)>,
    pub circles: HashSet<(u8, u8)>,
    pub squares: HashSet<(u8, u8)>,
    pub selected: HashSet<(u8, u8)>,
    pub lines: HashSet<((u8, u8), (u8, u8))>,
    pub arrows: HashSet<((u8, u8), (u8, u8))>,
    pub dimmed: HashSet<(u8, u8)>,
    pub labels: HashMap<(u8, u8), String>,
}

impl Goban {
    const DEFAULT_HOSHIS: [(u8, u8); 0] = [];
    const NINE_HOSHIS: [(u8, u8); 4] = [(2, 2), (2, 6), (6, 2), (6, 6)];
    const THIRTEEN_HOSHIS: [(u8, u8); 5] = [(3, 3), (3, 9), (6, 6), (9, 3), (9, 9)];
    const NINETEEN_HOSHIS: [(u8, u8); 9] = [
        (3, 3),
        (3, 9),
        (3, 15),
        (9, 3),
        (9, 9),
        (9, 15),
        (15, 3),
        (15, 9),
        (15, 15),
    ];

    pub fn new(board_size: (u8, u8)) -> Self {
        Self {
            size: board_size,
            stones: HashMap::new(),
            move_numbers: HashMap::new(),
            move_number: 0,
            black_captures: 0,
            white_captures: 0,
            marks: HashSet::new(),
            triangles: HashSet::new(),
            circles: HashSet::new(),
            squares: HashSet::new(),
            selected: HashSet::new(),
            lines: HashSet::new(),
            arrows: HashSet::new(),
            dimmed: HashSet::new(),
            labels: HashMap::new(),
        }
    }

    pub fn from_sgf_node(sgf_node: &SgfNode<go::Prop>) -> Result<Self, Box<dyn std::error::Error>> {
        let board_size = get_board_size(&sgf_node);
        let mut goban = Goban::new(board_size);
        goban.process_node(&sgf_node)?;

        Ok(goban)
    }

    pub fn stones(&self) -> impl Iterator<Item = Stone> {
        self.stones
            .iter()
            .map(|(point, color)| Stone {
                x: point.0,
                y: point.1,
                color: *color,
            })
            .collect::<Vec<Stone>>()
            .into_iter()
    }

    pub fn process_node(
        &mut self,
        sgf_node: &SgfNode<go::Prop>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.marks.clear();
        self.triangles.clear();
        self.circles.clear();
        self.squares.clear();
        self.selected.clear();
        self.dimmed.clear();
        self.labels.clear();
        self.lines.clear();
        self.arrows.clear();
        for prop in sgf_node.properties() {
            match prop {
                go::Prop::B(go::Move::Move(point)) => {
                    if !self.is_tt_pass(*point) {
                        self.play_stone(Stone::new(point.x, point.y, StoneColor::Black))?;
                    }
                }
                go::Prop::W(go::Move::Move(point)) => {
                    if !self.is_tt_pass(*point) {
                        self.play_stone(Stone::new(point.x, point.y, StoneColor::White))?;
                    }
                }
                go::Prop::AB(points) => {
                    for point in points.iter() {
                        self.add_stone(Stone::new(point.x, point.y, StoneColor::Black))?;
                    }
                }
                go::Prop::AW(points) => {
                    for point in points.iter() {
                        self.add_stone(Stone::new(point.x, point.y, StoneColor::White))?;
                    }
                }
                go::Prop::AE(points) => {
                    for point in points.iter() {
                        self.clear_point((point.x, point.y));
                    }
                }
                go::Prop::MN(num) => self.set_move_number(*num as u64),
                go::Prop::MA(points) => self.marks = points.iter().map(|p| (p.x, p.y)).collect(),
                go::Prop::TR(points) => {
                    self.triangles = points.iter().map(|p| (p.x, p.y)).collect()
                }
                go::Prop::CR(points) => self.circles = points.iter().map(|p| (p.x, p.y)).collect(),
                go::Prop::SQ(points) => self.squares = points.iter().map(|p| (p.x, p.y)).collect(),
                go::Prop::SL(points) => self.selected = points.iter().map(|p| (p.x, p.y)).collect(),
                go::Prop::DD(points) => self.dimmed = points.iter().map(|p| (p.x, p.y)).collect(),
                go::Prop::LB(labels) => {
                    self.labels = labels
                        .iter()
                        .map(|(p, t)| ((p.x, p.y), t.to_string()))
                        .collect()
                }
                go::Prop::LN(pairs) => {
                    self.lines = pairs
                        .iter()
                        .map(|(p1, p2)| ((p1.x, p1.y), (p2.x, p2.y)))
                        .collect()
                }
                go::Prop::AR(pairs) => {
                    self.arrows = pairs
                        .iter()
                        .map(|(p1, p2)| ((p1.x, p1.y), (p2.x, p2.y)))
                        .collect()
                }
                _ => {}
            }
        }

        Ok(())
    }

    pub fn add_stone(&mut self, stone: Stone) -> Result<(), GobanError> {
        if stone.x > self.size.0 || stone.y > self.size.1 {
            return Err(GobanError::InvalidMoveError);
        }
        let key = (stone.x, stone.y);
        if self.stones.contains_key(&key) {
            return Err(GobanError::InvalidMoveError);
        }
        self.stones.insert(key, stone.color);

        Ok(())
    }

    pub fn play_stone(&mut self, stone: Stone) -> Result<(), GobanError> {
        self.add_stone(stone)?;
        let opponent_color = match stone.color {
            StoneColor::Black => StoneColor::White,
            StoneColor::White => StoneColor::Black,
        };
        // Remove any neighboring groups with no liberties.
        let key = (stone.x, stone.y);
        for neighbor in self.neighbors(key) {
            if let Some(color) = self.stones.get(&neighbor) {
                if *color == opponent_color {
                    self.process_captures(neighbor);
                }
            }
        }
        // Now remove the played stone if still neccessary
        self.process_captures(key);
        self.move_number += 1;
        (*self
            .move_numbers
            .entry((stone.x, stone.y))
            .or_insert_with(Vec::new))
        .push(self.move_number);

        Ok(())
    }

    pub fn clear_point(&mut self, point: (u8, u8)) {
        self.stones.remove(&point);
    }

    pub fn set_move_number(&mut self, num: u64) {
        self.move_number = num;
    }

    pub fn hoshi_points(&self) -> impl Iterator<Item = &(u8, u8)> {
        match self.size {
            (9, 9) => Self::NINE_HOSHIS.iter(),
            (13, 13) => Self::THIRTEEN_HOSHIS.iter(),
            (19, 19) => Self::NINETEEN_HOSHIS.iter(),
            _ => Self::DEFAULT_HOSHIS.iter(),
        }
    }

    fn neighbors(&self, point: (u8, u8)) -> impl Iterator<Item = (u8, u8)> {
        let (x, y) = point;
        let mut neighbors = vec![];
        if x < self.size.0 - 1 {
            neighbors.push((x + 1, y));
        }
        if x > 0 {
            neighbors.push((x - 1, y));
        }
        if y < self.size.1 - 1 {
            neighbors.push((x, y + 1));
        }
        if y > 0 {
            neighbors.push((x, y - 1));
        }

        neighbors.into_iter()
    }

    fn process_captures(&mut self, start_point: (u8, u8)) {
        let group_color = match self.stones.get(&start_point) {
            Some(color) => color,
            None => return,
        };
        let mut group = HashSet::new();
        let mut to_process = VecDeque::new();
        to_process.push_back(start_point);
        while let Some(p) = to_process.pop_back() {
            group.insert(p);
            for neighbor in self.neighbors(p) {
                if group.contains(&neighbor) {
                    continue;
                }
                match self.stones.get(&neighbor) {
                    None => return,
                    Some(c) if c == group_color => {
                        to_process.push_back(neighbor);
                    }
                    _ => {}
                }
            }
        }
        match group_color {
            StoneColor::Black => self.black_captures += group.len() as u64,
            StoneColor::White => self.white_captures += group.len() as u64,
        }
        for stone in group {
            self.stones.remove(&stone);
        }
    }

    fn is_tt_pass(&self, point: go::Point) -> bool {
        point.x == 19 && point.y == 19 && self.size.0 < 20 && self.size.1 < 20
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum StoneColor {
    Black,
    White,
}

#[derive(Copy, Clone, Debug)]
pub struct Stone {
    pub x: u8,
    pub y: u8,
    pub color: StoneColor,
}

impl Stone {
    pub fn new(x: u8, y: u8, color: StoneColor) -> Stone {
        Stone { x, y, color }
    }
}

fn get_board_size(sgf_node: &SgfNode<go::Prop>) -> (u8, u8) {
    match sgf_node.get_property("SZ") {
        Some(go::Prop::SZ(size)) => *size,
        None => (19, 19),
        Some(_) => unreachable!(),
    }
}

#[derive(Debug)]
pub enum GobanError {
    InvalidMoveError,
}

impl std::fmt::Display for GobanError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            GobanError::InvalidMoveError => write!(f, "Invalid move"),
        }
    }
}

impl std::error::Error for GobanError {}
