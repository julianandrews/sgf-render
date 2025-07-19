use std::collections::{HashMap, HashSet, VecDeque};

use sgf_parse::{go, ParseOptions, SgfNode};

use crate::errors::GobanError;
use crate::render::{NodeDescription, NodeNumber};
use crate::sgf_traversal::variation_nodes;

pub struct Goban {
    size: (u8, u8),
    stones: HashMap<(u8, u8), StoneColor>,
    stones_before_move: HashMap<u64, HashSet<Stone>>,
    moves: Vec<(u64, Stone)>,
    move_number: u64,
    marks: HashSet<(u8, u8)>,
    triangles: HashSet<(u8, u8)>,
    circles: HashSet<(u8, u8)>,
    squares: HashSet<(u8, u8)>,
    selected: HashSet<(u8, u8)>,
    lines: HashSet<((u8, u8), (u8, u8))>,
    arrows: HashSet<((u8, u8), (u8, u8))>,
    dimmed: HashSet<(u8, u8)>,
    labels: HashMap<(u8, u8), String>,
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

    pub fn from_sgf(
        sgf: &str,
        node_description: &NodeDescription,
        strict: bool,
    ) -> Result<Self, GobanError> {
        let parse_options = ParseOptions {
            lenient: !strict,
            ..Default::default()
        };
        let collection = sgf_parse::parse_with_options(sgf, &parse_options)?;
        let root_node = collection
            .get(node_description.game_number as usize)
            .ok_or(GobanError::MissingGame)?
            .as_go_node()?;
        let board_size = get_board_size(root_node)?;
        let mut goban = Goban::new(board_size);
        let nodes = variation_nodes(root_node, node_description.variation)?;
        let mut node_count = 0;
        for node in nodes {
            goban.process_node(node.sgf_node)?;
            node_count += 1;
            if let NodeNumber::Number(n) = node_description.node_number {
                if node_count > n {
                    break;
                }
            }
        }
        if let NodeNumber::Number(n) = node_description.node_number {
            if n >= node_count {
                return Err(GobanError::InsufficientSgfNodes);
            }
        }
        Ok(goban)
    }

    pub fn stones(&self) -> impl Iterator<Item = Stone> + '_ {
        self.stones.iter().map(|(point, color)| Stone {
            x: point.0,
            y: point.1,
            color: *color,
        })
    }

    pub fn stones_before_move(&self, move_number: u64) -> Box<dyn Iterator<Item = Stone> + '_> {
        let stones = self.stones_before_move.get(&move_number);
        match stones {
            Some(stones) => Box::new(stones.iter().copied()),
            None => Box::new(self.stones.iter().map(|(point, color)| Stone {
                x: point.0,
                y: point.1,
                color: *color,
            })),
        }
    }

    pub fn stone_color(&self, x: u8, y: u8) -> Option<StoneColor> {
        self.stones.get(&(x, y)).copied()
    }

    pub fn moves(&self) -> impl Iterator<Item = (u64, Stone)> + '_ {
        self.moves.iter().copied()
    }

    pub fn hoshi_points(&self) -> impl Iterator<Item = (u8, u8)> {
        match self.size {
            (9, 9) => Self::NINE_HOSHIS.iter().copied(),
            (13, 13) => Self::THIRTEEN_HOSHIS.iter().copied(),
            (19, 19) => Self::NINETEEN_HOSHIS.iter().copied(),
            _ => Self::DEFAULT_HOSHIS.iter().copied(),
        }
    }

    pub fn size(&self) -> (u8, u8) {
        self.size
    }

    pub fn marks(&self) -> impl Iterator<Item = (u8, u8)> + '_ {
        self.marks.iter().copied()
    }

    pub fn triangles(&self) -> impl Iterator<Item = (u8, u8)> + '_ {
        self.triangles.iter().copied()
    }

    pub fn circles(&self) -> impl Iterator<Item = (u8, u8)> + '_ {
        self.circles.iter().copied()
    }

    pub fn squares(&self) -> impl Iterator<Item = (u8, u8)> + '_ {
        self.squares.iter().copied()
    }

    pub fn selected(&self) -> impl Iterator<Item = (u8, u8)> + '_ {
        self.selected.iter().copied()
    }

    pub fn dimmed(&self) -> impl Iterator<Item = (u8, u8)> + '_ {
        self.dimmed.iter().copied()
    }

    pub fn lines(&self) -> impl Iterator<Item = ((u8, u8), (u8, u8))> + '_ {
        self.lines.iter().copied()
    }

    pub fn arrows(&self) -> impl Iterator<Item = ((u8, u8), (u8, u8))> + '_ {
        self.arrows.iter().copied()
    }

    pub fn labels(&self) -> impl Iterator<Item = (&(u8, u8), &String)> {
        self.labels.iter()
    }

    fn new(board_size: (u8, u8)) -> Self {
        Self {
            size: board_size,
            stones: HashMap::new(),
            stones_before_move: HashMap::new(),
            moves: Vec::new(),
            move_number: 0,
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

    fn process_node(&mut self, sgf_node: &SgfNode<go::Prop>) -> Result<(), GobanError> {
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

    fn add_stone(&mut self, stone: Stone) -> Result<(), GobanError> {
        if stone.x > self.size.0 || stone.y > self.size.1 {
            return Err(GobanError::InvalidMove);
        }
        let key = (stone.x, stone.y);
        self.stones.insert(key, stone.color);

        Ok(())
    }

    fn play_stone(&mut self, stone: Stone) -> Result<(), GobanError> {
        self.stones_before_move.insert(
            self.move_number,
            self.stones
                .iter()
                .map(|(point, color)| Stone {
                    x: point.0,
                    y: point.1,
                    color: *color,
                })
                .collect(),
        );
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
        self.moves.push((self.move_number, stone));

        Ok(())
    }

    fn clear_point(&mut self, point: (u8, u8)) {
        self.stones.remove(&point);
    }

    fn set_move_number(&mut self, num: u64) {
        self.move_number = num;
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
        for stone in group {
            self.stones.remove(&stone);
        }
    }

    fn is_tt_pass(&self, point: go::Point) -> bool {
        point.x == 19 && point.y == 19 && self.size.0 < 20 && self.size.1 < 20
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum StoneColor {
    Black,
    White,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
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

fn get_board_size(sgf_node: &SgfNode<go::Prop>) -> Result<(u8, u8), GobanError> {
    match sgf_node.get_property("SZ") {
        Some(go::Prop::SZ((x, y))) if *x <= 52 && *y <= 52 => Ok((*x, *y)),
        None => Ok((19, 19)),
        _ => Err(GobanError::InvalidSzProperty),
    }
}

#[cfg(test)]
mod tests {
    use crate::errors::GobanError;

    use super::{get_board_size, Goban};

    #[test]
    fn play_over_existing_stone() {
        let result = Goban::from_sgf("(;AB[ac];B[ac])", &Default::default(), true);
        assert!(result.is_ok());
    }

    #[test]
    fn invalid_sz() {
        let collection = sgf_parse::go::parse("(;SZ[foo])").unwrap();
        let result = get_board_size(&collection[0]);
        assert!(matches!(result, Err(GobanError::InvalidSzProperty)));
    }

    #[test]
    fn strict_parsing_fails_for_bad_sgf() {
        let result = Goban::from_sgf("(;AB[ac];B[ac]", &Default::default(), true);
        assert!(!result.is_ok());
    }

    #[test]
    fn lenient_parsing_works_for_bad_sgf() {
        let result = Goban::from_sgf("(;AB[ac];B[ac]", &Default::default(), false);
        assert!(result.is_ok());
    }
}
