use std::path::PathBuf;

use clap::builder::styling::{AnsiColor, Styles};
use clap::Parser;

use crate::errors::UsageError;
use crate::render::{
    BoardSideSet, GeneratedStyle, GobanRange, MoveNumberOptions, NodeDescription, RenderOptions,
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
    #[clap(subcommand)]
    pub command: Option<Command>,
    /// SGF file to read from [default: read from stdin].
    #[arg(value_name = "FILE", global = true)]
    pub infile: Option<PathBuf>,
    /// Output file [default: write to stdout].
    #[arg(short, long, value_name = "FILE")]
    pub outfile: Option<PathBuf>,
    /// Output format.
    #[arg(short = 'f', long = "format", default_value = "svg")]
    #[cfg_attr(not(feature = "png"), arg(hide = true))]
    pub output_format: OutputFormat,
    #[clap(flatten)]
    pub render_args: RenderArgs,
}

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    /// Print a tree of the SGF's variations
    Query,
}

#[derive(Debug, Parser)]
pub struct RenderArgs {
    #[clap(flatten)]
    node_description: NodeDescription,
    /// Width of the output image in pixels.
    #[arg(
        short = 'w',
        long = "width",
        value_name = "WIDTH",
        default_value_t = 800.0
    )]
    viewbox_width: f64,
    /// Draw only enough of the board to hold all the stones (with 1 space padding).
    #[arg(short, long, conflicts_with = "range")]
    shrink_wrap: bool,
    /// Range to draw as a pair of corners (e.g. 'cc-ff').
    #[arg(short, long)]
    range: Option<GobanRange>,
    /// Style to use.
    #[arg(long = "style", value_name = "STYLE", default_value = "simple")]
    generated_style: GeneratedStyle,
    /// Custom style `toml` file. Conflicts with '--style'. See the README for details.
    #[arg(long, value_name = "FILE", conflicts_with = "generated_style")]
    custom_style: Option<PathBuf>,
    /// Draw move numbers (may replace other markup).
    #[arg(long, require_equals=true, num_args = 0..=1, value_name = "RANGE", default_missing_value = "1")]
    move_numbers: Option<MoveNumberRange>,
    /// Number to start counting move numbers from (requires --move-numbers).
    #[arg(
        long,
        value_name = "NUM",
        default_value_t = 1,
        requires = "move_numbers"
    )]
    move_numbers_from: u64,
    /// Sides to draw position labels on.
    #[arg(long, value_name = "SIDES", default_value = "nw")]
    label_sides: BoardSideSet,
    /// Don't draw position labels.
    #[arg(long, conflicts_with = "label_sides")]
    no_board_labels: bool,
    /// Don't draw SGF marks.
    #[clap(long = "no-marks", action = clap::ArgAction::SetFalse)]
    draw_marks: bool,
    /// Don't draw SGF triangles.
    #[clap(long = "no-triangles", action = clap::ArgAction::SetFalse)]
    draw_triangles: bool,
    /// Don't draw SGF circles.
    #[clap(long = "no-circles", action = clap::ArgAction::SetFalse)]
    draw_circles: bool,
    /// Don't draw SGF squares.
    #[clap(long = "no-squares", action = clap::ArgAction::SetFalse)]
    draw_squares: bool,
    /// Don't draw SGF selected.
    #[clap(long = "no-selected", action = clap::ArgAction::SetFalse)]
    draw_selected: bool,
    /// Don't draw SGF dimmed.
    #[clap(long = "no-dimmed", action = clap::ArgAction::SetFalse)]
    draw_dimmed: bool,
    /// Don't draw SGF labels.
    #[clap(long = "no-labels", action = clap::ArgAction::SetFalse)]
    draw_labels: bool,
    /// Don't draw SGF lines.
    #[clap(long = "no-lines", action = clap::ArgAction::SetFalse)]
    draw_lines: bool,
    /// Don't draw SGF arrows.
    #[clap(long = "no-arrows", action = clap::ArgAction::SetFalse)]
    draw_arrows: bool,
    /// Don't draw any markup on points.
    #[clap(long)]
    no_point_markup: bool,
    /// Generate a kifu.
    #[clap(long)]
    kifu: bool,
}

impl RenderArgs {
    /// Map RenderArgs to RenderOptions.
    pub fn options(&self, output_format: &OutputFormat) -> Result<RenderOptions, UsageError> {
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
                    .map_err(|e| UsageError::StyleReadError(e.into()))?;
                toml::from_str(&data).map_err(|e| UsageError::StyleReadError(e.into()))?
            }
            None => self.generated_style.style().clone(),
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

        if output_format == &OutputFormat::Text {
            if self.kifu {
                return Err(UsageError::InvalidTextOutputOption("Kifu mode".to_owned()));
            }
            if move_number_options.is_some() {
                return Err(UsageError::InvalidTextOutputOption(
                    "Move numbers".to_owned(),
                ));
            }
        }

        Ok(RenderOptions {
            node_description: self.node_description,
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

#[derive(Debug, Clone, Copy, clap::ValueEnum, Eq, PartialEq)]
pub enum OutputFormat {
    Svg,
    Text,
    #[cfg(feature = "png")]
    Png,
}

#[derive(Debug, Clone, Copy)]
struct MoveNumberRange {
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
