use crate::MakeSvgOptions;
use std::path::PathBuf;

const DEFAULT_MOVE_NUMBER: u64 = 1;

pub fn parse_args(
    opts: &getopts::Options,
    args: &Vec<String>,
) -> Result<SgfRenderArgs, UsageError> {
    // TODO: writeme!
    let infile = Some(PathBuf::from("/home/julian/Downloads/tsumego/prob0001.sgf"));
    let outfile = Some(PathBuf::from("/tmp/out.svg"));
    let options: MakeSvgOptions = Default::default();

    let matches = opts
        .parse(&args[1..])
        .map_err(|_| UsageError::ArgumentParseError)?;
    let move_number = matches
        .opt_str("m")
        .map(|c| c.parse())
        .unwrap_or(Ok(DEFAULT_MOVE_NUMBER))
        .map_err(|_| UsageError::ArgumentParseError)?;
    let print_help = matches.opt_present("h");

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
    opts.optflag("h", "help", "Display this help and exit");
    opts.optopt(
        "m",
        "move-number",
        &format!("Move number to render (default {})", DEFAULT_MOVE_NUMBER,),
        "MOVE_NUMBER",
    );

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
        // TODO: writeme!
        match self {
            UsageError::ArgumentParseError => write!(f, "Failed to parse arguments."),
        }
    }
}

impl ::std::error::Error for UsageError {}
