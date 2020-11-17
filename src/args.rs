use crate::lib::{GobanRange, GobanStyle, MakeSvgOptions};
use std::path::PathBuf;

const DEFAULT_NODE_NUM: u64 = 1;
const DEFAULT_FIRST_MOVE_NUM: u64 = 1;
const DEFAULT_WIDTH: u32 = 800;

pub fn parse_args(
    opts: &getopts::Options,
    args: &Vec<String>,
) -> Result<SgfRenderArgs, UsageError> {
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
        .map(|c| c.parse())
        .unwrap_or(Ok(DEFAULT_NODE_NUM))
        .map_err(|_| UsageError::InvalidNodeNumber)?;
    let render_labels = !matches.opt_present("no-labels");
    let viewbox_width = matches
        .opt_str("w")
        .map(|c| c.parse::<u32>())
        .unwrap_or(Ok(DEFAULT_WIDTH))
        .map_err(|_| UsageError::InvalidWidth)? as f64;
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
        .unwrap_or("default".to_string())
        .as_str()
    {
        "default" => Ok(GobanStyle::Default),
        "simple" => Ok(GobanStyle::Simple),
        "minimalist" => Ok(GobanStyle::Minimalist),
        _ => Err(UsageError::InvalidStyle),
    }?;
    let render_move_numbers = matches.opt_present("move-numbers");
    let first_move_number = matches
        .opt_str("first-move-number")
        .map(|c| c.parse())
        .unwrap_or(Ok(DEFAULT_FIRST_MOVE_NUM))
        .map_err(|_| UsageError::InvalidFirstMoveNumber)?;

    // There isn't really a clean way to render both markup and move numbers that I can see.
    let render_marks = !render_move_numbers && !matches.opt_present("no-marks");
    let render_triangles = !render_move_numbers && !matches.opt_present("no-triangles");
    let render_circles = !render_move_numbers && !matches.opt_present("no-circles");
    let render_squares = !render_move_numbers && !matches.opt_present("no-squares");
    let render_selected = !render_move_numbers && !matches.opt_present("no-selected");
    let render_dimmed = !render_move_numbers && !matches.opt_present("no-dimmed");

    let options = MakeSvgOptions {
        goban_range,
        render_labels,
        render_move_numbers,
        render_marks,
        render_triangles,
        render_circles,
        render_squares,
        render_selected,
        render_dimmed,
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
        "Style to use. One of 'default', 'simple' or 'minimalist'",
        "STYLE",
    );
    opts.optflag(
        "",
        "move-numbers",
        "Draw move numbers (disables other markup).",
    );
    opts.optflag("", "no-marks", "Don't render SGF marks.");
    opts.optflag("", "no-triangles", "Don't render SGF triangles.");
    opts.optflag("", "no-circles", "Don't render SGF circles.");
    opts.optflag("", "no-squares", "Don't render SGF squares.");
    opts.optflag("", "no-selected", "Don't render SGF selected.");
    opts.optflag("", "no-dimmed", "Don't render SGF dimmmed.");
    opts.optopt(
        "",
        "first-move-number",
        "First move number to draw if using --move-numbers",
        "NUM",
    );
    opts.optflag("", "no-labels", "Don't render labels on the diagram");
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
    let parse_byte = |b: u8| {
        if b < b'a' || b > b'z' {
            Err(UsageError::InvalidRange)
        } else {
            Ok(b - b'a')
        }
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
