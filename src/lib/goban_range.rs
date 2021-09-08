use std::collections::HashSet;
use std::ops::Range;

use super::{Goban, MakeSvgError, MakeSvgOptions};

#[derive(Debug, Clone)]
pub enum GobanRange {
    ShrinkWrap,
    FullBoard,
    Ranged(Range<u8>, Range<u8>),
}

impl GobanRange {
    pub fn get_ranges(
        &self,
        goban: &Goban,
        options: &MakeSvgOptions,
    ) -> Result<(Range<u8>, Range<u8>), MakeSvgError> {
        match self {
            Self::FullBoard => Ok((0..goban.size.0, 0..goban.size.1)),
            Self::ShrinkWrap => {
                let mut points: HashSet<_> = goban
                    .stones(options.kifu_mode)
                    .map(|s| (s.x, s.y))
                    .collect();
                if options.draw_marks {
                    points.extend(goban.marks.iter().copied());
                }
                if options.draw_triangles {
                    points.extend(goban.triangles.iter().copied());
                }
                if options.draw_circles {
                    points.extend(goban.circles.iter().copied());
                }
                if options.draw_squares {
                    points.extend(goban.squares.iter().copied());
                }
                if options.draw_selected {
                    points.extend(goban.selected.iter().copied());
                }
                if options.draw_labels {
                    points.extend(goban.labels.keys().copied());
                }
                if options.draw_lines {
                    points.extend(goban.lines.iter().flat_map(|&(p1, p2)| vec![p1, p2]))
                }
                if options.draw_squares {
                    points.extend(goban.arrows.iter().flat_map(|&(p1, p2)| vec![p1, p2]))
                }
                // Don't necessarily include dimmed points!
                let x_start = {
                    let p = points
                        .iter()
                        .map(|&(x, _)| x)
                        .min()
                        .unwrap_or(0)
                        .saturating_sub(1);
                    if p == 1 {
                        0 // Include nearby board edge.
                    } else {
                        p
                    }
                };
                let x_end = {
                    let p = points
                        .iter()
                        .map(|&(x, _)| (x + 2).min(goban.size.0))
                        .max()
                        .unwrap_or(goban.size.0);
                    if p == goban.size.0 - 1 {
                        goban.size.0 // Include nearby board edge.
                    } else {
                        p
                    }
                };
                let y_start = {
                    let p = points
                        .iter()
                        .map(|&(_, y)| y)
                        .min()
                        .unwrap_or(0)
                        .saturating_sub(1);
                    if p == 1 {
                        0 // Include nearby board edge.
                    } else {
                        p
                    }
                };
                let y_end = {
                    let p = points
                        .iter()
                        .map(|&(_, y)| (y + 2).min(goban.size.1))
                        .max()
                        .unwrap_or(goban.size.1);
                    if p == goban.size.1 - 1 {
                        goban.size.1 // Include nearby board edge.
                    } else {
                        p
                    }
                };
                Ok((x_start..x_end, y_start..y_end))
            }
            Self::Ranged(a, b) => {
                if a.end > goban.size.0 || b.end > goban.size.1 {
                    Err(MakeSvgError::InvalidRange)
                } else {
                    Ok((a.clone(), b.clone()))
                }
            }
        }
    }
}
