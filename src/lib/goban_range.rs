use std::collections::HashSet;
use std::ops::Range;

use crate::errors::{MakeSvgError, UsageError};
use crate::goban::Goban;
use crate::make_svg::MakeSvgOptions;

#[derive(Debug, Clone, Default)]
pub enum GobanRange {
    ShrinkWrap,
    #[default]
    FullBoard,
    Ranged(Range<u8>, Range<u8>),
}

impl GobanRange {
    pub fn get_ranges(
        &self,
        goban: &Goban,
        options: &MakeSvgOptions,
    ) -> Result<(Range<u8>, Range<u8>), MakeSvgError> {
        let goban_size = goban.size();
        match self {
            Self::FullBoard => Ok((0..goban_size.0, 0..goban_size.1)),
            Self::ShrinkWrap => {
                let mut points: HashSet<_> = goban.stones().map(|s| (s.x, s.y)).collect();
                if options.draw_marks {
                    points.extend(goban.marks());
                }
                if options.draw_triangles {
                    points.extend(goban.triangles());
                }
                if options.draw_circles {
                    points.extend(goban.circles());
                }
                if options.draw_squares {
                    points.extend(goban.squares());
                }
                if options.draw_selected {
                    points.extend(goban.selected());
                }
                if options.draw_labels {
                    points.extend(goban.labels().map(|(p, _)| p))
                }
                if options.draw_lines {
                    points.extend(goban.lines().flat_map(|(p1, p2)| vec![p1, p2]))
                }
                if options.draw_arrows {
                    points.extend(goban.arrows().flat_map(|(p1, p2)| vec![p1, p2]))
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
                        .map(|&(x, _)| (x + 2).min(goban_size.0))
                        .max()
                        .unwrap_or(goban_size.0);
                    if p == goban_size.0 - 1 {
                        goban_size.0 // Include nearby board edge.
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
                        .map(|&(_, y)| (y + 2).min(goban_size.1))
                        .max()
                        .unwrap_or(goban_size.1);
                    if p == goban_size.1 - 1 {
                        goban_size.1 // Include nearby board edge.
                    } else {
                        p
                    }
                };
                Ok((x_start..x_end, y_start..y_end))
            }
            Self::Ranged(a, b) => {
                if a.end > goban_size.0 || b.end > goban_size.1 {
                    Err(MakeSvgError::InvalidRange)
                } else {
                    Ok((a.clone(), b.clone()))
                }
            }
        }
    }
}

impl std::str::FromStr for GobanRange {
    type Err = UsageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parse_byte = |b: u8| match b {
            b'a'..=b'z' => Ok(b - b'a'),
            _ => Err(UsageError::InvalidRange),
        };

        let s = s.as_bytes();
        if s.len() != 5 || s[2] != b'-' {
            return Err(UsageError::InvalidRange);
        }
        Ok(GobanRange::Ranged(
            parse_byte(s[0])?..parse_byte(s[3])? + 1,
            parse_byte(s[1])?..parse_byte(s[4])? + 1,
        ))
    }
}
