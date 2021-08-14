use std::collections::HashSet;
use std::ops::Range;

use super::{Goban, GobanSVGError};

#[derive(Debug)]
pub enum GobanRange {
    ShrinkWrap,
    FullBoard,
    Ranged(Range<u8>, Range<u8>),
}

impl GobanRange {
    pub fn get_ranges(&self, goban: &Goban) -> Result<(Range<u8>, Range<u8>), GobanSVGError> {
        match self {
            Self::FullBoard => Ok((0..goban.size.0, 0..goban.size.1)),
            Self::ShrinkWrap => {
                let points: HashSet<(u8, u8)> = goban
                    .stones()
                    .map(|s| (s.x, s.y))
                    .chain(goban.marks.iter().copied())
                    .chain(goban.triangles.iter().copied())
                    .chain(goban.circles.iter().copied())
                    .chain(goban.squares.iter().copied())
                    .chain(goban.selected.iter().copied())
                    .chain(goban.labels.keys().copied())
                    .chain(goban.lines.iter().flat_map(|&(p1, p2)| vec![p1, p2]))
                    .chain(goban.arrows.iter().flat_map(|&(p1, p2)| vec![p1, p2]))
                    // Don't necessarily include dimmed points!
                    .collect();
                let x_start = points
                    .iter()
                    .map(|&(x, _)| x)
                    .min()
                    .unwrap_or(0)
                    .saturating_sub(1);
                let x_end = points
                    .iter()
                    .map(|&(x, _)| (x + 2).min(goban.size.0))
                    .max()
                    .unwrap_or(goban.size.0);
                let y_start = points
                    .iter()
                    .map(|&(_, y)| y)
                    .min()
                    .unwrap_or(0)
                    .saturating_sub(1);
                let y_end = points
                    .iter()
                    .map(|&(_, y)| (y + 2).min(goban.size.1))
                    .max()
                    .unwrap_or(goban.size.1);
                Ok((x_start..x_end, y_start..y_end))
            }
            Self::Ranged(a, b) => {
                if a.end > goban.size.0 || b.end > goban.size.1 {
                    Err(GobanSVGError::InvalidRange)
                } else {
                    Ok((a.clone(), b.clone()))
                }
            }
        }
    }
}
