use crate::lib::{GobanRange, MakeSvgOptions};
use std::path::PathBuf;

const DEFAULT_MOVE_NUMBER: u64 = 1;
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
    let move_number = matches
        .opt_str("m")
        .map(|c| c.parse())
        .unwrap_or(Ok(DEFAULT_MOVE_NUMBER))
        .map_err(|_| UsageError::InvalidMoveNumber)?;
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

    let options = MakeSvgOptions {
        goban_range,
        render_labels,
        viewbox_width,
    };

    Ok(SgfRenderArgs {
        infile,
        outfile,
        move_number,
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
        "m",
        "move-num",
        &format!("Move number to render (default {})", DEFAULT_MOVE_NUMBER,),
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
    opts.optflag("", "no-labels", "Don't render labels on the diagram");
    opts.optflag("h", "help", "Display this help and exit");

    opts
}

#[derive(Debug)]
pub struct SgfRenderArgs {
    pub infile: Option<PathBuf>,
    pub outfile: Option<PathBuf>,
    pub move_number: u64,
    pub options: MakeSvgOptions,
    pub print_help: bool,
}

#[derive(Debug)]
pub enum UsageError {
    FailedToParse,
    TooManyArguments,
    InvalidMoveNumber,
    InvalidWidth,
    OverspecifiedRange,
    InvalidRange,
}

impl std::fmt::Display for UsageError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            UsageError::FailedToParse => write!(f, "Failed to parse arguments."),
            UsageError::TooManyArguments => write!(f, "Too many arguments."),
            UsageError::InvalidMoveNumber => write!(f, "Invalid move number."),
            UsageError::InvalidWidth => write!(f, "Invalid width."),
            UsageError::OverspecifiedRange => write!(f, "Specify only '-r' or '-s'"),
            UsageError::InvalidRange => write!(f, "Invalid range."),
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
