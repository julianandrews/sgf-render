use std::path::PathBuf;

use clap::builder::styling::{AnsiColor, Styles};
use clap::Parser;

use super::{
    BoardSide, BoardSideSet, GobanRange, MakeSvgOptions, MoveNumberOptions, NodeDescription,
    GENERATED_STYLES,
};

// clap v3 styling
const CLAP_STYLES: Styles = Styles::styled()
    .header(AnsiColor::Yellow.on_default())
    .usage(AnsiColor::Green.on_default())
    .literal(AnsiColor::Green.on_default())
    .placeholder(AnsiColor::Green.on_default());

#[derive(Debug, Parser)]
#[clap(version, about, styles=CLAP_STYLES)]
pub struct SgfRenderArgs {
    /// SGF file to render (defaults to stdin).
    #[arg(value_name = "FILE")]
    pub infile: Option<PathBuf>,
    /// Output file. SVG and PNG formats supported.
    #[arg(short, long, value_name = "FILE")]
    pub outfile: Option<PathBuf>,
    #[clap(flatten)]
    pub make_svg_args: MakeSvgArgs,
}

impl MakeSvgArgs {
    pub fn options(&self) -> Result<MakeSvgOptions, UsageError> {
        let node_description = match &self.node_description {
            Some(node_description) => node_description.clone(),
            None => NodeDescription::default(self.kifu),
        };

        let goban_range = if self.shrink_wrap {
            GobanRange::ShrinkWrap
        } else if let Some(range) = &self.range {
            range.clone()
        } else {
            GobanRange::FullBoard
        };

        let style = match &self.custom_style {
            Some(filename) => {
                let data = std::fs::read_to_string(filename)
                    .map_err(|e| UsageError::InvalidStyleFile(e.into()))?;
                toml::from_str(&data).map_err(|e| UsageError::InvalidStyleFile(e.into()))?
            }
            None => GENERATED_STYLES
                .get(self.style.to_string().as_str())
                .ok_or(UsageError::InvalidStyle)?
                .clone(),
        };

        let count_from = self.move_numbers_from;
        let move_number_options = if let Some(range) = self.move_numbers {
            Some(MoveNumberOptions {
                start: range.start,
                end: range.end,
                count_from,
            })
        } else if self.kifu {
            Some(MoveNumberOptions {
                start: 1,
                end: None,
                count_from,
            })
        } else {
            None
        };

        let no_point_markup = self.no_point_markup;
        let label_sides = if self.no_board_labels {
            BoardSideSet::default()
        } else {
            self.label_sides
        };

        Ok(MakeSvgOptions {
            node_description,
            goban_range,
            style,
            viewbox_width: self.viewbox_width,
            label_sides,
            move_number_options,
            draw_marks: self.draw_marks && !no_point_markup,
            draw_triangles: self.draw_triangles && !no_point_markup,
            draw_circles: self.draw_circles && !no_point_markup,
            draw_squares: self.draw_squares && !no_point_markup,
            draw_selected: self.draw_selected && !no_point_markup,
            draw_dimmed: self.draw_dimmed && !no_point_markup,
            draw_labels: self.draw_labels && !no_point_markup,
            draw_lines: self.draw_lines && !no_point_markup,
            draw_arrows: self.draw_arrows && !no_point_markup,
            kifu_mode: self.kifu,
        })
    }
}

#[derive(Debug, Parser)]
pub struct MakeSvgArgs {
    /// Node to render. For simple use provide a number or `last` to render
    /// the last node. See the README for more detail.
    #[arg(short, long = "node", value_name = "PATH_SPEC")]
    pub node_description: Option<NodeDescription>,
    /// Width of the output image in pixels.
    #[arg(
        short = 'w',
        long = "width",
        value_name = "WIDTH",
        default_value_t = 800.0
    )]
    pub viewbox_width: f64,
    /// Draw only enough of the board to hold all the stones (with 1 space padding).
    #[arg(short, long, conflicts_with = "range")]
    pub shrink_wrap: bool,
    /// Range to draw as a pair of corners (e.g. 'cc-ff').
    #[arg(short, long)]
    pub range: Option<GobanRange>,
    /// Style to use. One of 'simple', 'fancy' or 'minimalist'.
    #[arg(long, value_name = "STYLE", default_value = "simple")]
    pub style: Style,
    /// Custom style to use. Overrides '--style'. See the README for details.
    #[arg(long, value_name = "FILE", conflicts_with = "style")]
    pub custom_style: Option<PathBuf>,
    /// Draw move numbers (may replace other markup).
    #[arg(long, require_equals=true, num_args = 0..=1, value_name = "RANGE", default_missing_value = "1")]
    pub move_numbers: Option<MoveNumberRange>,
    /// Number to start counting move numbers from (requires --move-numbers).
    #[arg(
        long,
        value_name = "NUM",
        default_value_t = 1,
        requires = "move_numbers"
    )]
    pub move_numbers_from: u64,
    /// Sides to draw position labels on.
    #[arg(long, value_name = "SIDES", default_value = "nw")]
    pub label_sides: BoardSideSet,
    /// Don't draw position labels.
    #[arg(long, conflicts_with = "label_sides")]
    pub no_board_labels: bool,
    /// Don't draw SGF marks.
    #[clap(long = "no-marks", action = clap::ArgAction::SetFalse)]
    pub draw_marks: bool,
    /// Don't draw SGF triangles.
    #[clap(long = "no-triangles", action = clap::ArgAction::SetFalse)]
    pub draw_triangles: bool,
    /// Don't draw SGF circles.
    #[clap(long = "no-circles", action = clap::ArgAction::SetFalse)]
    pub draw_circles: bool,
    /// Don't draw SGF squares.
    #[clap(long = "no-squares", action = clap::ArgAction::SetFalse)]
    pub draw_squares: bool,
    /// Don't draw SGF selected.
    #[clap(long = "no-selected", action = clap::ArgAction::SetFalse)]
    pub draw_selected: bool,
    /// Don't draw SGF dimmed.
    #[clap(long = "no-dimmed", action = clap::ArgAction::SetFalse)]
    pub draw_dimmed: bool,
    /// Don't draw SGF labels.
    #[clap(long = "no-labels", action = clap::ArgAction::SetFalse)]
    pub draw_labels: bool,
    /// Don't draw SGF lines.
    #[clap(long = "no-lines", action = clap::ArgAction::SetFalse)]
    pub draw_lines: bool,
    /// Don't draw SGF arrows.
    #[clap(long = "no-arrows", action = clap::ArgAction::SetFalse)]
    pub draw_arrows: bool,
    /// Don't draw any markup on points.
    #[clap(long)]
    pub no_point_markup: bool,
    /// Generate a kifu.
    #[clap(long)]
    pub kifu: bool,
}

#[derive(Debug, Clone)]
pub enum Style {
    Simple,
    Minimalist,
    Fancy,
}

impl std::str::FromStr for Style {
    type Err = UsageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "simple" => Ok(Style::Simple),
            "minimalist" => Ok(Style::Minimalist),
            "fancy" => Ok(Style::Fancy),
            _ => Err(UsageError::InvalidStyle),
        }
    }
}

impl std::fmt::Display for Style {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Style::Simple => write!(f, "simple"),
            Style::Minimalist => write!(f, "minimalist"),
            Style::Fancy => write!(f, "fancy"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MoveNumberRange {
    start: u64,
    end: Option<u64>,
}

impl std::str::FromStr for MoveNumberRange {
    type Err = UsageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<_> = s.splitn(2, '-').collect();
        let start = parts[0]
            .parse()
            .map_err(|_| UsageError::InvalidFirstMoveNumber)?;
        let end = parts
            .get(1)
            .map(|end| end.parse())
            .transpose()
            .map_err(|_| UsageError::InvalidLastMoveNumber)?;
        Ok(MoveNumberRange { start, end })
    }
}

#[derive(Debug)]
pub enum UsageError {
    InvalidRange,
    InvalidStyle,
    InvalidStyleFile(Box<dyn std::error::Error>),
    InvalidFirstMoveNumber,
    InvalidLastMoveNumber,
    InvalidBoardSides,
}

impl std::fmt::Display for UsageError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            UsageError::InvalidRange => write!(f, "Invalid range."),
            UsageError::InvalidStyle => write!(f, "Invalid style."),
            UsageError::InvalidStyleFile(e) => write!(f, "Failed to read style file: {}", e),
            UsageError::InvalidFirstMoveNumber => write!(f, "Invalid first move number."),
            UsageError::InvalidLastMoveNumber => write!(f, "Invalid last move number."),
            UsageError::InvalidBoardSides => write!(f, "Invalid board sides."),
        }
    }
}

impl std::error::Error for UsageError {}
unsafe impl Send for UsageError {}
unsafe impl Sync for UsageError {}

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
