#![feature(str_split_once)]

mod args;
mod lib;

use lib::Goban;
use std::error::Error;
use std::path::PathBuf;
use svg::node::element::SVG;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let opts = args::build_opts();
    let parsed_args = match args::parse_args(&opts, &args) {
        Ok(args) => args,
        Err(error) => {
            eprintln!("{}", error);
            args::print_usage(&args[0], &opts);
            std::process::exit(1);
        }
    };
    if parsed_args.print_help {
        args::print_usage(&args[0], &opts);
        return;
    }

    let goban = match load_goban(&parsed_args.infile, parsed_args.move_number) {
        Ok(goban) => goban,
        Err(e) => {
            eprintln!("Failed to load SGF node: {}", e);
            std::process::exit(1);
        }
    };

    let document = match lib::make_svg(&goban, &parsed_args.options) {
        Ok(document) => document,
        Err(e) => {
            eprintln!("Failed to generate SVG: {}", e);
            std::process::exit(1);
        }
    };

    let result: Result<(), Box<dyn Error>> = match parsed_args.outfile {
        Some(filename) => write_to_file(&filename, &document).into(),
        None => svg::write(std::io::stdout(), &document).map_err(|e| e.into()),
    };
    if let Err(e) = result {
        eprintln!("Failed to write output: {}", e);
        std::process::exit(1);
    }
}

fn load_goban(infile: &Option<PathBuf>, move_number: u64) -> Result<Goban, Box<dyn Error>> {
    let mut sgf_node = &get_sgf_root(infile)?;

    let mut goban = Goban::from_sgf_node(&sgf_node)?;
    for _ in 1..move_number {
        sgf_node = sgf_node
            .children()
            .next()
            .ok_or(SgfRenderError::InsufficientSgfNodes)?;
        goban.process_node(sgf_node)?;
    }

    Ok(goban)
}

fn get_sgf_root(infile: &Option<PathBuf>) -> Result<sgf_parse::SgfNode, Box<dyn Error>> {
    let mut reader: Box<dyn std::io::Read> = match infile {
        Some(filename) => Box::new(std::io::BufReader::new(std::fs::File::open(&filename)?)),
        None => Box::new(std::io::stdin()),
    };
    let mut text = String::new();
    reader.read_to_string(&mut text)?;
    let collection = sgf_parse::parse(&text)?;
    collection
        .into_iter()
        .next()
        .ok_or(Box::new(SgfRenderError::NoSgfNodes))
}

fn write_to_file(outfile: &std::path::PathBuf, document: &SVG) -> Result<(), Box<dyn Error>> {
    match outfile.extension().and_then(std::ffi::OsStr::to_str) {
        Some("svg") => svg::save(&outfile, document)?,
        Some("png") => save_png(&outfile, document)?,
        _ => Err(SgfRenderError::UnsupportedFileExtension)?,
    }
    Ok(())
}

#[cfg(feature = "png")]
fn save_png(outfile: &PathBuf, document: &SVG) -> Result<(), Box<dyn Error>> {
    let s = document.to_string();
    let mut fontdb = usvg::fontdb::Database::new();
    let font_data = include_bytes!("../data/Roboto-Bold.ttf").to_vec();
    fontdb.load_font_data(font_data);
    let tree = usvg::Tree::from_str(
        &s,
        &usvg::Options {
            fontdb,
            ..Default::default()
        },
    )?;
    let img =
        resvg::render(&tree, usvg::FitTo::Original, None).ok_or(SgfRenderError::PNGRenderFailed)?;
    img.save_png(outfile)?;
    Ok(())
}

#[cfg(not(feature = "png"))]
fn save_png(_outfile: &PathBuf, _document: &SVG) -> Result<(), Box<dyn Error>> {
    Err(SgfRenderError::NoPngSupport)?
}

#[derive(Debug)]
enum SgfRenderError {
    NoSgfNodes,
    InsufficientSgfNodes,
    PNGRenderFailed,
    UnsupportedFileExtension,
    NoPngSupport,
}

impl std::fmt::Display for SgfRenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoSgfNodes => write!(f, "No sgf nodes found in input."),
            Self::InsufficientSgfNodes => write!(f, "Insufficient SGF nodes for move number."),
            Self::PNGRenderFailed => write!(f, "Rendering png failed."),
            Self::UnsupportedFileExtension => write!(f, "Unsupported file extension."),
            Self::NoPngSupport => write!(f, "Compiled without png support."),
        }
    }
}

impl Error for SgfRenderError {}
