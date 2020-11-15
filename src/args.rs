use crate::lib::{GobanRange, MakeSvgOptions};
use std::path::PathBuf;

const DEFAULT_MOVE_NUMBER: u64 = 1;
const DEFAULT_WIDTH: u32 = 800;

pub fn parse_args(
    opts: &getopts::Options,
    args: &Vec<String>,
) -> Result<SgfRenderArgs, UsageError> {
    // TODO: Parse goban_range somehow.
    // TODO: Provide more granular error messages.
    let goban_range = GobanRange::ShrinkWrap;

    let matches = opts
        .parse(&args[1..])
        .map_err(|_| UsageError::ArgumentParseError)?;
    if matches.free.len() > 1 {
        return Err(UsageError::ArgumentParseError);
    }
    let infile = matches.free.first().map(PathBuf::from);
    let outfile = matches.opt_str("o").map(PathBuf::from);
    let move_number = matches
        .opt_str("m")
        .map(|c| c.parse())
        .unwrap_or(Ok(DEFAULT_MOVE_NUMBER))
        .map_err(|_| UsageError::ArgumentParseError)?;
    let render_labels = !matches.opt_present("no-labels");
    let viewbox_width = matches
        .opt_str("w")
        .map(|c| c.parse::<u32>())
        .unwrap_or(Ok(DEFAULT_WIDTH))
        .map_err(|_| UsageError::ArgumentParseError)? as f64;
    let print_help = matches.opt_present("h");

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
    let brief = format!("Usage: {} [options]", program);
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
        "move-number",
        &format!("Move number to render (default {})", DEFAULT_MOVE_NUMBER,),
        "MOVE_NUMBER",
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
    ArgumentParseError,
}

impl std::fmt::Display for UsageError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            UsageError::ArgumentParseError => write!(f, "Failed to parse arguments."),
        }
    }
}

impl ::std::error::Error for UsageError {}
