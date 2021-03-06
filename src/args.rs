use crate::lib::{GobanRange, GobanStyle, MakeSvgOptions};
use std::path::PathBuf;

const DEFAULT_NODE_NUM: u64 = 1;
const DEFAULT_FIRST_MOVE_NUM: u64 = 1;
const DEFAULT_WIDTH: u32 = 800;

pub fn parse(opts: &getopts::Options, args: &[String]) -> Result<SgfRenderArgs, UsageError> {
    let matches = opts
        .parse(&args[1..])
        .map_err(|_| UsageError::FailedToParse)?;
    if matches.free.len() > 1 {
        return Err(UsageError::TooManyArguments);
    }
    let infile = matches.free.first().map(PathBuf::from);
    let outfile = matches.opt_str("o").map(PathBuf::from);
    let node_number = matches
        .opt_str("n")
        .map_or(Ok(DEFAULT_NODE_NUM), |c| c.parse())
        .map_err(|_| UsageError::InvalidNodeNumber)?;
    let draw_board_labels = !matches.opt_present("no-board-labels");
    let viewbox_width = f64::from(
        matches
            .opt_str("w")
            .map_or(Ok(DEFAULT_WIDTH), |c| c.parse::<u32>())
            .map_err(|_| UsageError::InvalidWidth)?,
    );
    let print_help = matches.opt_present("h");
    let goban_range = {
        if matches.opt_present("shrink-wrap") && matches.opt_present("r") {
            return Err(UsageError::OverspecifiedRange);
        }

        if matches.opt_present("shrink-wrap") {
            GobanRange::ShrinkWrap
        } else {
            match matches.opt_str("r") {
                Some(s) => parse_point_pair(&s)?,
                None => GobanRange::FullBoard,
            }
        }
    };
    let style = match matches
        .opt_str("style")
        .unwrap_or_else(|| "simple".to_string())
        .as_str()
    {
        "simple" => Ok(GobanStyle::Simple),
        "fancy" => Ok(GobanStyle::Fancy),
        "minimalist" => Ok(GobanStyle::Minimalist),
        _ => Err(UsageError::InvalidStyle),
    }?;
    let draw_move_numbers = matches.opt_present("move-numbers");
    let first_move_number = matches
        .opt_str("first-move-number")
        .map_or(Ok(DEFAULT_FIRST_MOVE_NUM), |c| c.parse())
        .map_err(|_| UsageError::InvalidFirstMoveNumber)?;

    // There isn't really a clean way to draw both markup and move numbers that I can see.
    let draw_marks = !draw_move_numbers && !matches.opt_present("no-marks");
    let draw_triangles = !draw_move_numbers && !matches.opt_present("no-triangles");
    let draw_circles = !draw_move_numbers && !matches.opt_present("no-circles");
    let draw_squares = !draw_move_numbers && !matches.opt_present("no-squares");
    let draw_selected = !draw_move_numbers && !matches.opt_present("no-selected");
    let draw_dimmed = !draw_move_numbers && !matches.opt_present("no-dimmed");
    let draw_labels = !draw_move_numbers && !matches.opt_present("no-labels");
    let draw_lines = !draw_move_numbers && !matches.opt_present("no-lines");
    let draw_arrows = !draw_move_numbers && !matches.opt_present("no-arrows");

    let options = MakeSvgOptions {
        goban_range,
        draw_board_labels,
        draw_move_numbers,
        draw_marks,
        draw_triangles,
        draw_circles,
        draw_squares,
        draw_selected,
        draw_dimmed,
        draw_labels,
        draw_lines,
        draw_arrows,
        first_move_number,
        viewbox_width,
        style,
    };

    Ok(SgfRenderArgs {
        infile,
        outfile,
        node_number,
        options,
        print_help,
    })
}

pub fn print_usage(program: &str, opts: &getopts::Options) {
    let brief = format!("Usage: {} [FILE] [options]", program);
    print!("{}", opts.usage(&brief));
}

pub fn build_opts() -> getopts::Options {
    let mut opts = getopts::Options::new();
    opts.optopt(
        "o",
        "outfile",
        "Output file. SVG and PNG formats supported.",
        "FILE",
    );
    opts.optopt(
        "n",
        "node-num",
        &format!(
            "Node number to render (default {}). Note that SGFs \
            may have nodes without moves.",
            DEFAULT_NODE_NUM,
        ),
        "NUM",
    );
    opts.optopt(
        "w",
        "width",
        &format!(
            "Width of the output image in pixels (default {})",
            DEFAULT_WIDTH,
        ),
        "WIDTH",
    );
    opts.optflag(
        "s",
        "shrink-wrap",
        "Draw only enough of the board to hold all the stones (with 1 space padding)",
    );
    opts.optopt(
        "r",
        "range",
        "Range to draw as a pair of corners (e.g. 'cc-ff')",
        "RANGE",
    );
    opts.optopt(
        "",
        "style",
        "Style to use. One of 'simple', 'fancy' or 'minimalist'",
        "STYLE",
    );
    opts.optflag(
        "",
        "move-numbers",
        "Draw move numbers (disables other markup).",
    );
    opts.optflag("", "no-board-labels", "Don't draw position labels.");
    opts.optflag("", "no-marks", "Don't draw SGF marks.");
    opts.optflag("", "no-triangles", "Don't draw SGF triangles.");
    opts.optflag("", "no-circles", "Don't draw SGF circles.");
    opts.optflag("", "no-squares", "Don't draw SGF squares.");
    opts.optflag("", "no-selected", "Don't draw SGF selected.");
    opts.optflag("", "no-dimmed", "Don't draw SGF dimmmed.");
    opts.optflag("", "no-labels", "Don't draw SGF labels.");
    opts.optflag("", "no-lines", "Don't draw SGF lines.");
    opts.optflag("", "no-arrows", "Don't draw SGF arrows.");
    opts.optopt(
        "",
        "first-move-number",
        "First move number to draw if using --move-numbers",
        "NUM",
    );
    opts.optflag("h", "help", "Display this help and exit");

    opts
}

#[derive(Debug)]
pub struct SgfRenderArgs {
    pub infile: Option<PathBuf>,
    pub outfile: Option<PathBuf>,
    pub node_number: u64,
    pub options: MakeSvgOptions,
    pub print_help: bool,
}

#[derive(Debug)]
pub enum UsageError {
    FailedToParse,
    TooManyArguments,
    InvalidNodeNumber,
    InvalidWidth,
    OverspecifiedRange,
    InvalidRange,
    InvalidStyle,
    InvalidFirstMoveNumber,
}

impl std::fmt::Display for UsageError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            UsageError::FailedToParse => write!(f, "Failed to parse arguments."),
            UsageError::TooManyArguments => write!(f, "Too many arguments."),
            UsageError::InvalidNodeNumber => write!(f, "Invalid node number."),
            UsageError::InvalidWidth => write!(f, "Invalid width."),
            UsageError::OverspecifiedRange => write!(f, "Specify only '-r' or '-s'"),
            UsageError::InvalidRange => write!(f, "Invalid range."),
            UsageError::InvalidStyle => write!(f, "Invalid style."),
            UsageError::InvalidFirstMoveNumber => write!(f, "Invalid first move number."),
        }
    }
}

impl ::std::error::Error for UsageError {}

fn parse_point_pair(s: &str) -> Result<GobanRange, UsageError> {
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
